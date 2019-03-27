// -*- mode: Javascript; indent-tabs-mode: nil; -*-

"use strict";

import * as common from "./common.js";

export function init(config) {
    console.log("md mode");

    let url = config.url;
    let mdUrl = common.mdUrlFromUrl(config.url);

    console.log(`md url: ${mdUrl}`);

    window.addEventListener("scroll", function() {
        saveScrollPos(config);
    });

    common.insertRenderedFile(config, mdUrl, function() {
        common.showPage();
        restoreScrollPos(config);
    });
}

function saveScrollPos(config) {
    sessionStorage.setItem(`y:${config.url}`, window.scrollY);
}

function restoreScrollPos(config) {
    let y = sessionStorage.getItem(`y:${config.url}`);
    if (y) {
        window.scrollTo(0, y);
    }
}
