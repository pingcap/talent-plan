// -*- mode: Javascript; indent-tabs-mode: nil; -*-

"use strict";

import * as mdMode from "./md-mode.js";
import * as lessonSlidesMode from "./lesson-slides-mode.js";

if (document.readyState === "complete") {
    init();
} else {
    window.addEventListener("load", init);
}

function init() {
    let url = document.ORIGINAL_URI;
    let pageType = getPageType(url);

    let contentElement = document.getElementById("content");

    let config = {
        url: url,
        baseUrl: document.BASE_URI,
        pageType: pageType,
        contentElement: contentElement,
    };

    addCssForPageType(config);
    dispatchInitForPageType(config);
}

function getPageType(url) {
    if (url.endsWith("#/")) {
        url = url.slice(0, -2);
    }
        
    let type = "unknown";
    if (url.indexOf(".slides.") != -1) {
        type = "lesson-slides";
    } else {
        type = "md";
    }

    return type;
}

function addCssForPageType(config) {
    if (config.pageType === "lesson-slides") {
        return;
    }

    let head = document.querySelector("head");
    let link = document.createElement("link");
    link.setAttribute("rel", "stylesheet");
    link.setAttribute("href", "css/text.css");
    head.appendChild(link);
}

function dispatchInitForPageType(config) {
    let mainModule = null;
    if (config.pageType === "md") {
        mainModule = mdMode;
    } else if (config.pageType === "lesson-slides") {
        mainModule = lessonSlidesMode;
    }
    mainModule.init(config);
}
