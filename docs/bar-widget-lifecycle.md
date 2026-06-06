# Bar Widget Lifecycle

This diagram traces how `cargo run` turns app config into rendered bar widgets, then how service state reaches those mounted widgets after startup.

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

## Key Handoff Points

- `Shell` owns which bar windows exist.
- `BarLayout` translates config strings into `WidgetInstance`s.
- `registry::widget_by_id` turns a widget id like `clock` into a registered widget factory.
- `Bar::mount_region` is where widget factories are built and appended into GTK containers.
- `ShellMsg::ItemStateChanged` is the service-to-shell state path.
- `BarMsg::ItemStateChanged` is the shell-to-bar state path.
- `BarWidgetRuntime::update` is the bar-to-widget render-update path.
