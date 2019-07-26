use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

use dces::prelude::{Entity, EntityComponentManager, System};

use crate::{prelude::*, shell::WindowShell, tree::Tree, utils::*};

/// The `RenderSystem` iterates over all visual widgets and used its render objects to draw them on the screen.
pub struct RenderSystem {
    pub render_objects: Rc<RefCell<BTreeMap<Entity, Box<dyn RenderObject>>>>,
    pub shell: Rc<RefCell<WindowShell<WindowAdapter>>>,
    pub update: Rc<Cell<bool>>,
    pub running: Rc<Cell<bool>>,
}

impl System<Tree> for RenderSystem {
    fn run(&self, tree: &Tree, ecm: &mut EntityComponentManager) {
        if !self.update.get() || tree.parent.is_empty() || !self.running.get() {
            return;
        }

        #[cfg(feature = "debug")]
        let debug = true;
        #[cfg(not(feature = "debug"))]
        let debug = false;

        let theme = ecm.borrow_component::<Theme>(tree.root).unwrap().0.clone();

        let mut hidden_parents: HashSet<Entity> = HashSet::new();

        let mut offsets = BTreeMap::new();
        offsets.insert(tree.root, (0.0, 0.0));

        // render window background
        // let selector = SelectorValue::new().with("window");
        // if let Some(background) = render_context.theme.brush("background", &selector) {
        //     render_context.renderer.render(background.into())
        // }

        for node in tree.into_iter() {
            let mut global_position = Point::default();

            if let Some(parent) = tree.parent[&node] {
                if let Some(offset) = offsets.get(&parent) {
                    global_position = Point::new(offset.0, offset.1);
                }
            }

            // Hide all children of a hidden parent
            if let Some(parent) = tree.parent[&node] {
                if hidden_parents.contains(&parent) {
                    hidden_parents.insert(node);
                    continue;
                }
            }

            // hide hidden widget
            if let Ok(visibility) = ecm.borrow_component::<Visibility>(node) {
                if visibility.0 != VisibilityValue::Visible {
                    hidden_parents.insert(node);
                    continue;
                }
            }

            if let Some(render_object) = self.render_objects.borrow().get(&node) {
                render_object.render(
                    &mut Context::new(node, ecm, tree, &mut self.shell.borrow_mut(), &theme),
                    &global_position,
                );
            }

            // render debug border for each widget
            if debug {
                if let Ok(bounds) = ecm.borrow_component::<Bounds>(node) {
                    let selector = Selector::from("debug-border");
                    let brush = theme.brush("border-color", &selector.0).unwrap();
                    self.shell
                        .borrow_mut()
                        .render_context_2_d()
                        .set_stroke_style(brush);
                    self.shell.borrow_mut().render_context_2_d().stroke_rect(
                        global_position.x + bounds.x(),
                        global_position.y + bounds.y(),
                        bounds.width(),
                        bounds.height(),
                    );
                }
            }

            let mut global_pos = (0.0, 0.0);

            if let Ok(bounds) = ecm.borrow_component::<Bounds>(node) {
                global_pos = (
                    global_position.x + bounds.x(),
                    global_position.y + bounds.y(),
                );
                offsets.insert(node, global_pos);
            }

            if let Ok(g_pos) = ecm.borrow_mut_component::<Point>(node) {
                g_pos.x = global_pos.0;
                g_pos.y = global_pos.1;
            }
        }

        // self.update.set(false);
    }
}