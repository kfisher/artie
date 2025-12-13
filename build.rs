// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "compiled.gresource",
    );
}
