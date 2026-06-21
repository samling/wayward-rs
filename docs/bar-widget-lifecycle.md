# Bar Widget Lifecycle

This document traces how Wayward turns `config.toml` into mounted bar widgets, then how widget state and widget actions move through the running application.

There are two related flows:

- Startup and creation: config is resolved into `WidgetInstance`s, then each widget factory builds a mounted runtime.
- Ongoing messages: services send state into mounted runtimes, and widgets send actions back out to services or shell-level handlers.

## High-Level Lifecycle

```mermaid
flowchart TD
    run["cargo run"] --> main["main"]
    main --> config_files["ensure config files"]
    main --> services["initialize services"]
    main --> app["run Shell app"]

    app --> shell_init["initialize Shell"]
    shell_init --> load_config["load app config"]
    shell_init --> initial_states["collect initial item states"]
    shell_init --> reconcile["reconcile bars"]

    reconcile --> desired["compute desired bars"]
    desired --> launch["launch missing bars"]
    launch --> bar_init["build BarInit"]

    bar_init --> layout["build BarLayout"]
    layout --> regions["read start center end"]
    regions --> parse_items["parse configured items"]
    parse_items --> parse_item["parse one item"]
    parse_item --> split_ref["split widget reference"]
    parse_item --> registry["look up widget factory"]
    registry --> widget_instance["create WidgetInstance"]

    bar_init --> bar_launch["launch Bar component"]
    bar_launch --> bar_init_fn["initialize Bar"]
    bar_init_fn --> bar_view["create region boxes"]
    bar_init_fn --> mount_layout["mount layout"]

    widget_instance --> mount_region
    mount_layout --> mount_region["mount region"]
    mount_region --> build["build widget runtime"]
    build --> runtime["widget runtime"]
    runtime --> root["get root widget"]
    root --> append["append root to container"]
    append --> mounted["store MountedWidget"]

    shell_init --> start_all["start services and widget tasks"]
    start_all --> widget_start["start widget service"]
    widget_start --> shell_msg["send item state to Shell"]
    shell_msg --> shell_update["update Shell"]
    shell_update --> shell_store["store Shell item state"]
    shell_update --> fanout["forward item state to bars"]
    fanout --> bar_update["update Bar"]
    bar_update --> bar_store["store Bar item state"]
    bar_update --> apply_state["apply state to mounted widgets"]
    apply_state --> match_id["match by widget id"]
    match_id --> runtime_update["update widget runtime"]
    runtime_update --> widget_render["render widget-specific state"]
```

## Startup Sequence

```mermaid
sequenceDiagram
    autonumber

    participant Main as main.rs
    participant Config as config
    participant Services as ShellServices
    participant App as RelmApp
    participant Shell as Shell
    participant Bars as shell::bars
    participant Layout as BarLayout
    participant Registry as registry
    participant Bar as Bar
    participant Widget as BarWidget
    participant Runtime as BarWidgetRuntime

    Main->>Config: ensure_config_files()
    Main->>Services: init_shell_services()
    Main->>App: run Shell
    App->>Shell: init(ShellInit)

    Shell->>Config: AppConfig::load()
    Shell->>Services: initial_item_states()
    Shell->>Bars: reconcile_bars()

    Bars->>Bars: desired_bars()
    Bars->>Layout: BarLayout::from_config()
    Layout->>Registry: widget_by_id(widget_type)
    Registry-->>Layout: BarWidget
    Layout-->>Bars: start, center, end WidgetInstances

    Bars->>Bar: launch BarInit
    Bar->>Bar: initial_model()
    Bar->>Bar: mount_layout()
    Bar->>Widget: build(instance, context)
    Widget-->>Bar: Box<dyn BarWidgetRuntime>
    Bar->>Runtime: root()
    Runtime-->>Bar: gtk::Widget
    Bar->>Bar: append root to region
    Bar->>Bar: store MountedWidget

    Shell->>Services: start_all()
    Services->>Widget: widget.start(sender, services)
```

## Ongoing Workflow

```mermaid
flowchart TD
    subgraph ConfigFlow["Config and layout flow"]
        config_file["config.toml"] --> app_config["AppConfig"]
        app_config --> bar_layout["BarLayout"]
        bar_layout --> widget_instance["WidgetInstance"]
        widget_instance --> widget_build["BarWidget::build"]
        widget_build --> mounted_widget["MountedWidget"]
    end

    subgraph Mounted["Mounted widget rendezvous point"]
        mounted_widget --> runtime["BarWidgetRuntime"]
        runtime --> relm_component["widget Relm component"]
        runtime --> gtk_root["GTK root widget"]
    end

    subgraph StateFlow["Service state flow"]
        service_watch["widget service or watcher"] --> shell_state["ShellMsg::ItemStateChanged"]
        shell_state --> shell_store["Shell stores latest BarItemState"]
        shell_store --> bar_state["BarMsg::ItemStateChanged"]
        bar_state --> bar_store["Bar stores latest BarItemState"]
        bar_store --> match_widget["match MountedWidget by widget_id"]
        match_widget --> runtime_update["BarWidgetRuntime::update"]
        runtime_update --> relm_component
    end

    subgraph ActionFlow["Widget action flow"]
        relm_component --> widget_event["BarMsg::WidgetEvent"]
        widget_event --> shell_action{"Shell-level action?"}
        shell_action -->|OpenSettings| shell_output["ShellMsg::OpenSettings"]
        shell_action -->|widget action| registry_dispatch["registry::handle_widget_event"]
        registry_dispatch --> widget_service["widget.handle_event"]
        widget_service --> command_or_service["command, D-Bus call, or service mutation"]
    end

    subgraph HotReloadFlow["Config hot reload flow"]
        file_watch["file watch"] --> config_changed["ShellMsg::ConfigChanged"]
        config_changed --> diff["ConfigChanges::between"]
        diff -->|bars or widgets changed| reconcile["Shell::reconcile_bars"]
        reconcile --> layout_changed["BarMsg::LayoutChanged"]
        layout_changed --> preserve_or_replace["Bar::reconcile_region"]
        preserve_or_replace --> mounted_widget
    end
```

## Important Types

`BarWidget` is the static widget definition. It knows the widget id, how to build a runtime, how to provide an initial state, how to start background work, and how to handle widget events.

`WidgetInstance` is a config-resolved widget entry. It contains the configured id, widget type, optional instance name, resolved config table, and the registered `BarWidget`.

`BarWidgetRuntime` is the live handle returned by `BarWidget::build`. The bar stores this in `MountedWidget` and uses it to get the GTK root and push state updates into the widget.

`MountedWidget` is the point where creation flow and message flow meet. It stores the `WidgetInstance`, the widget id, the bar region, and the live runtime.

`BarItemState` is the service-to-render state envelope. Services and watchers send it to `Shell` with `ShellMsg::ItemStateChanged`.

`WidgetAction` is the widget-to-service command envelope. It is split into domain-specific action enums such as `VolumeAction`, `BrightnessAction`, and `NotificationAction`.

## Key Handoff Points

- `main.rs` initializes config files, services, CSS, and the Relm app.
- `Shell` owns which bar windows exist and forwards item state to each running bar.
- `shell::bars` reconciles configured bars against available monitors.
- `BarLayout` translates config strings into `WidgetInstance`s.
- `registry::widget_by_id` turns a widget id like `clock` into a registered widget factory.
- `Bar::mount_region` builds widget runtimes and appends their GTK roots into the start, center, or end container.
- `ShellMsg::ItemStateChanged` is the service-to-shell state path.
- `BarMsg::ItemStateChanged` is the shell-to-bar state path.
- `BarWidgetRuntime::update` is the bar-to-widget render update path.
- `BarMsg::WidgetEvent` is the widget-to-bar action path.
- `registry::handle_widget_event` routes widget actions back to the owning widget module.
