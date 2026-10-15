use super::*;
use bongocat_ui_protocol::{
    OnlineError, OnlinePhase, OnlineSettingsCapability, OnlineSettingsHandle, OnlineSnapshot,
};
use std::sync::{Arc, Mutex};

struct OnlinePhaseFixture(Arc<Mutex<OnlinePhase>>);

impl OnlineSettingsCapability for OnlinePhaseFixture {
    fn snapshot(&self) -> OnlineSnapshot {
        OnlineSnapshot {
            phase: *self.0.lock().unwrap(),
            ..OnlineSnapshot::default()
        }
    }

    fn join(&self, _: String) -> Result<(), OnlineError> {
        Ok(())
    }

    fn leave(&self) {}

    fn retry_rendering(&self) {}
}

struct OnlinePageHarness(Entity<SettingsView>);

impl Render for OnlinePageHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.0.update(cx, |view, cx| {
            view.sync_online_address(window, cx);
        });
        div().size_full().child(
            Settings::new("online-page-test")
                .sidebar_width(px(220.0))
                .with_group_variant(GroupBoxVariant::Outline)
                .page(super::super::online::page(
                    self.0.clone(),
                    SettingsLanguage::English,
                )),
        )
    }
}

#[gpui_kit::test]
fn online_address_is_wide_editable_and_keeps_its_draft(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (client, _endpoint) = crate::SettingsClient::bounded(4);
    let phase = Arc::new(Mutex::new(OnlinePhase::Offline));
    client.set_online_client(OnlineSettingsHandle::new(OnlinePhaseFixture(Arc::clone(
        &phase,
    ))));
    let built = Rc::new(RefCell::new(None));
    let capture = Rc::clone(&built);
    let (_, visual) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| {
            SettingsView::new(
                client,
                SettingsWindowSeed {
                    language: SettingsLanguage::English,
                    appearance_theme: SettingsTheme::System,
                },
                Rc::new(|_| {}),
                Rc::new(|_| {}),
                window,
                cx,
            )
        });
        capture.borrow_mut().replace(view.clone());
        Root::new(cx.new(|_| OnlinePageHarness(view)), window, cx)
    });
    let view = built.borrow_mut().take().unwrap();
    visual.simulate_resize(size(px(800.0), px(600.0)));
    visual.update(|window, cx| window.render_frame(cx));
    let field = rendered_bounds(visual, ElementId::from("online-address-field"));
    assert!(field.size.width > px(320.0), "the address needs a full row");
    assert!(field.left() >= px(0.0) && field.right() <= px(800.0));

    visual.simulate_click(field.center(), Modifiers::default());
    let address = "ws://203.0.113.10/bongocat";
    visual.simulate_input(address);
    assert_eq!(
        view.read_with(visual, |view, cx| view
            .online_address
            .read(cx)
            .value()
            .to_string()),
        address,
        "clicking and typing must edit the address before the first snapshot"
    );
    view.update(visual, |view, cx| {
        let mut snapshot = crate::tests::snapshot(1, false, true);
        snapshot.online_server_address = "ws://203.0.113.20/bongocat".into();
        view.snapshot = Some(snapshot);
        cx.notify();
    });
    visual.update(|window, cx| {
        view.update(cx, |view, cx| view.sync_online_address(window, cx));
        window.render_frame(cx);
    });
    assert_eq!(
        view.read_with(visual, |view, cx| view
            .online_address
            .read(cx)
            .value()
            .to_string()),
        address,
        "a late snapshot must not overwrite the typed draft"
    );

    for (index, current) in [
        OnlinePhase::Connecting,
        OnlinePhase::Connected,
        OnlinePhase::Reconnecting,
        OnlinePhase::Failed,
    ]
    .into_iter()
    .enumerate()
    {
        *phase.lock().unwrap() = current;
        view.update(visual, |_, cx| cx.notify());
        visual.update(|window, cx| window.render_frame(cx));
        visual.simulate_click(field.center(), Modifiers::default());
        visual.dispatch_action(gpui_kit::component::input::SelectAll);
        let pasted = format!("ws://203.0.113.{}:18080/bongocat", index + 30);
        visual.write_to_clipboard(gpui_kit::ClipboardItem::new_string(pasted.clone()));
        visual.dispatch_action(gpui_kit::component::input::Paste);
        assert_eq!(
            view.read_with(visual, |view, cx| view
                .online_address
                .read(cx)
                .value()
                .to_string()),
            pasted,
            "pasting an address must work in {current:?}"
        );
    }
}
