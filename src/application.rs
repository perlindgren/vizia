use std::{cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc};

use femtovg::{Align, Baseline, Canvas, Paint, Path, renderer::OpenGl};
use glutin::{ContextBuilder, event::VirtualKeyCode, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use morphorm::Units;

use crate::{CachedData, Color, Context, Data, Entity, Enviroment, Event, EventManager, IdManager, MouseButton, MouseButtonState, MouseState, Propagation, Style, Tree, TreeExt, WindowEvent, apply_hover, scan_to_code, style, vcode_to_code, vk_to_key};

static FONT: &[u8] = include_bytes!("Roboto-Regular.ttf");

pub struct Application {
    context: Context,
    builder: Option<Box<dyn Fn(&mut Context)>>,
}

impl Application {
    pub fn new<F>(builder: F) -> Self
    where F: 'static + Fn(&mut Context)
    {

        let mut cache = CachedData::default();
        cache.add(Entity::root());

        let mut context = Context {
            entity_manager: IdManager::new(),
            tree: Tree::new(),
            current: Entity::root(),
            count: 0,
            views: HashMap::new(),
            state: HashMap::new(),  
            data: Data::new(),
            style: Rc::new(RefCell::new(Style::default())),
            cache,
            enviroment: Enviroment::new(),
            event_queue: VecDeque::new(),
            mouse: MouseState::default(),
            hovered: Entity::root(),
            focused: Entity::root(),
            state_count: 0,
        };

        context.entity_manager.create();

        

        Self {
            context,
            builder: Some(Box::new(builder)),
        }
    }

    pub fn background_color(self, color: Color) -> Self {
        self.context.style.borrow_mut().background_color.insert(Entity::root(), color);

        self
    }

    pub fn locale(mut self, id: &str) -> Self {
        self.context.enviroment.set_locale(id);


        self
    }

    pub fn run(mut self) {

        let mut context = self.context;
        
        let event_loop = EventLoop::new();
        
        let handle = ContextBuilder::new()
            .build_windowed(WindowBuilder::new(), &event_loop)
            .expect("Failed to build windowed context");

        let handle = unsafe { handle.make_current().unwrap() };

        let renderer = OpenGl::new(|s| handle.context().get_proc_address(s) as *const _)
            .expect("Cannot create renderer");
        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

        let font = canvas.add_font_mem(FONT).expect("Failed to load font");

        let dpi_factor = handle.window().scale_factor();
        let size = handle.window().inner_size();

        let clear_color = context.style.borrow_mut().background_color.get(Entity::root()).cloned().unwrap_or_default();

        canvas.set_size(size.width as u32, size.height as u32, dpi_factor as f32);
        canvas.clear_rect(
            0,
            0,
            size.width as u32,
            size.height as u32,
            clear_color.into(),
        );

        context
            .cache
            .set_width(Entity::root(), 800.0);
        context
            .cache
            .set_height(Entity::root(), 600.0);

        context.style.borrow_mut().width.insert(Entity::root(), Units::Pixels(800.0));
        context.style.borrow_mut().height.insert(Entity::root(), Units::Pixels(600.0));

        let mut event_manager = EventManager::new();

        if let Some(builder) = self.builder.take() {
            (builder)(&mut context);

            self.builder = Some(builder);
        }

        let builder = self.builder.take();

        event_loop.run(move |event, _, control_flow|{
            *control_flow = ControlFlow::Wait;

            match event {
                glutin::event::Event::MainEventsCleared => {

                    if context.enviroment.needs_rebuild {
                        context.current = Entity::root();
                        context.count = 0;
                        if let Some(builder) = &builder {
                            (builder)(&mut context);
                        }
                        context.enviroment.needs_rebuild = false;
                    }

                    // Events
                    while !context.event_queue.is_empty() {
                        event_manager.flush_events(&mut context);
                    }

                    // Updates
                    for entity in context.tree.clone().into_iter() {
                        let mut observers = Vec::new();
                     
                        if let Some(model_list) = context.data.model_data.get(entity) {
                            for model in model_list.iter() {
                                //observers = model.update();
                                if model.is_dirty() {
                                    observers.extend(model.update().iter());
                                }
                            }
                        }

                        for observer in observers.iter() {
                            if let Some(mut view) = context.views.remove(observer) {
                                let prev = context.current;
                                context.current = *observer;
                                let prev_count = context.count;
                                context.count = 0;
                                view.body(&mut context);
                                context.current = prev;
                                context.count = prev_count;
                    
                
                                context.views.insert(*observer, view);
                            }
                        }

                        if let Some(model_list) = context.data.model_data.get_mut(entity) {
                            for model in model_list.iter_mut() {
                                model.reset();
                            }
                        }
                        
                    }

                    // Styling (TODO)

                    // Layout
                    morphorm::layout(&mut context.cache, &context.tree, &context.style.borrow());

                    handle.window().request_redraw();
                }

                glutin::event::Event::RedrawRequested(_) => {
                    // Redraw here
                    //println!("Redraw");
                    let clear_color = context.style.borrow_mut().background_color.get(Entity::root()).cloned().unwrap_or(Color::white());
                    canvas.clear_rect(
                        0,
                        0,
                        size.width as u32,
                        size.height as u32,
                        clear_color.into(),
                    );
                    for entity in context.tree.clone().into_iter() {
                        //println!("{}", debug(&mut context, entity));
                        let bounds = context.cache.get_bounds(entity);
                        let mut path = Path::new();
                        path.rect(bounds.x, bounds.y, bounds.w, bounds.h);

                        let background_color: femtovg::Color = context.style.borrow_mut().background_color.get(entity).cloned().unwrap_or_default().into();
                        canvas.fill_path(&mut path, Paint::color(background_color));
                        
                        if let Some(text) = context.style.borrow().text.get(entity) {
                            let mut paint = Paint::color(femtovg::Color::black());
                            paint.set_font(&[font]);
                            paint.set_text_align(Align::Center);
                            paint.set_text_baseline(Baseline::Middle);
                            canvas.fill_text(bounds.x + bounds.w / 2.0, bounds.y + bounds.h / 2.0, text, paint);
                        }
                    }

                    canvas.flush();
                    handle.swap_buffers().expect("Failed to swap buffers");
                }

                glutin::event::Event::WindowEvent {
                    window_id: _,
                    event,
                } => {
                    match event {
                        glutin::event::WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }

                        glutin::event::WindowEvent::CursorMoved {
                            device_id,
                            position,
                            modifiers
                        } => {

                            context.mouse.cursorx = position.x as f32;
                            context.mouse.cursory = position.y as f32;

                            apply_hover(&mut context);
                        }

                        glutin::event::WindowEvent::MouseInput {
                            device_id,
                            button,
                            state,
                            modifiers,
                        } => {
                            let button = match button {
                                glutin::event::MouseButton::Left => MouseButton::Left,
                                glutin::event::MouseButton::Right => MouseButton::Right,
                                glutin::event::MouseButton::Middle => MouseButton::Middle,
                                glutin::event::MouseButton::Other(val) => MouseButton::Other(val),
                            };

                            let state = match state {
                                glutin::event::ElementState::Pressed => MouseButtonState::Pressed,
                                glutin::event::ElementState::Released => MouseButtonState::Released,
                            };

                            match state {
                                MouseButtonState::Pressed => {
                                    context.event_queue.push_back(Event::new(WindowEvent::MouseDown(button)).target(context.hovered).propagate(Propagation::Up));
                                }

                                MouseButtonState::Released => {
                                    context.event_queue.push_back(Event::new(WindowEvent::MouseUp(button)).target(context.hovered).propagate(Propagation::Up));
                                }
                            }
                        }

                        glutin::event::WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic,
                        } => {
                            if input.virtual_keycode == Some(VirtualKeyCode::H) {
                                println!("Tree");
                                for entity in context.tree.into_iter() {
                                    println!("Entity: {} Parent: {:?} posx: {} posy: {} width: {} height: {}", entity, entity.parent(&context.tree), context.cache.get_posx(entity), context.cache.get_posy(entity), context.cache.get_width(entity), context.cache.get_height(entity));
                                }
                            }

                            let s = match input.state {
                                glutin::event::ElementState::Pressed => MouseButtonState::Pressed,
                                glutin::event::ElementState::Released => MouseButtonState::Released,
                            };

	                        // Prefer virtual keycodes to scancodes, as scancodes aren't uniform between platforms
	                        let code = if let Some(vkey) = input.virtual_keycode {
		                        vcode_to_code(vkey)
	                        } else {
		                        scan_to_code(input.scancode)
	                        };

                            let key = vk_to_key(
                                input.virtual_keycode.unwrap_or(VirtualKeyCode::NoConvert),
                            );

                            match s {
                                MouseButtonState::Pressed => {
                                    if context.focused != Entity::null() {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyDown(code, key))
                                                .target(context.focused)
                                                .propagate(Propagation::DownUp),
                                        );
                                    } else {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyDown(code, key))
                                                .target(context.hovered)
                                                .propagate(Propagation::DownUp),
                                        );
                                    }
                                }

                                MouseButtonState::Released => {
                                    if context.focused != Entity::null() {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyUp(code, key))
                                                .target(context.focused)
                                                .propagate(Propagation::DownUp),
                                        );
                                    } else {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyUp(code, key))
                                                .target(context.hovered)
                                                .propagate(Propagation::DownUp),
                                        );
                                    }
                                }
                            }
                        }

                        glutin::event::WindowEvent::ReceivedCharacter(character) => {
                            context.event_queue.push_back(
                                Event::new(WindowEvent::CharInput(character))
                                    .target(context.focused)
                                    .propagate(Propagation::Down),
                            );
                        }


                        _=> {}
                    }
                }

                _=> {}
            }
        });
    }
}

fn debug(cx: &mut Context, entity: Entity) -> String {
    if let Some(view) = cx.views.get(&entity) {
        view.debug(entity)
    } else {
        "None".to_string()
    }
}