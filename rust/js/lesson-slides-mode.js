// -*- mode: Javascript; indent-tabs-mode: nil; -*-

"use strict";

import * as common from "./common.js";

export function init(config) {

    let reveal_mod = "js/reveal.js/js/reveal";

    // This is a hack to keep the markdown module from attempting to require the
    // marked module synchronously, as if it were running under node. It is
    // reset after reveal is finished loading.
    define.amd = false;

    require([reveal_mod], function(_reveal) {

        console.log("holy reveal!");

        let reveal = window.Reveal;

        let mdUrl = common.mdUrlFromUrl(config.url);
        console.log(`md url: ${mdUrl}`);

        // Create the elements needed to run reveal.
        // For simplicity the reveal div is added directly as
        // the first child of the body element, with the nav
        // element positioned absolutely on top of it.

        let revealDiv = document.createElement("div");
        revealDiv.className = "reveal";
        let slidesDiv = document.createElement("div");
        slidesDiv.className = "slides";
        revealDiv.appendChild(slidesDiv);

        let mdSection = document.createElement("section");
        mdSection.setAttribute("data-markdown", mdUrl);
        mdSection.setAttribute("data-charset", "utf-8");
        mdSection.setAttribute("data-separator", "^\n\n\n\n")
        mdSection.setAttribute("data-separator-notes", "^<!-- text -->")
        slidesDiv.appendChild(mdSection);

        let body = document.querySelector("body");
        body.insertBefore(revealDiv, body.firstChild);

        let revealCfg = {
            history: true,
            controls: true,
            progress: true,
            center: true,
            dependencies: [
                { src: 'js/reveal.js/plugin/markdown/marked.js' },
                { src: 'js/reveal.js/plugin/markdown/markdown.js' },
                { src: 'js/reveal.js/plugin/notes/notes.js', async: true }
                //{ src: 'node_modules/reveal.js/plugin/highlight/highlight.js', async: true, callback: function() { fetchAllCode(); hljs.initHighlightingOnLoad(); addButtons(); } },
            ]
        };

        reveal.addEventListener("ready", function() {
            define.amd = true;

            common.rewriteUrls(revealDiv);
        });

        common.showPage();
        reveal.initialize(revealCfg);
    });
}

