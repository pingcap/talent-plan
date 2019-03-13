// -*- mode: Javascript; indent-tabs-mode: nil; -*-

"use strict";

import * as common from "./common.js";

export function init(config) {
    console.log("md mode");

    let url = config.url;
    let mdUrl = common.mdUrlFromUrl(config.url);

    console.log(`md url: ${mdUrl}`);

    common.insertRenderedFile(config, mdUrl);
}


