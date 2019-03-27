// -*- mode: Javascript; indent-tabs-mode: nil; -*-

"use strict";

let initialized = false;

export function mdUrlFromUrl(url) {
    let mdUrl = url;
    if (url.indexOf("index.html") != -1) {
        mdUrl = url.replace("index.html", "README.md");
    } else if (url.endsWith("/")) {
        mdUrl = url + "README.md";
    } else if (url.indexOf(".slides.html") != -1) {
        mdUrl = url.replace(".slides.html", ".md");
    } else if (url.indexOf(".html") != -1) {
        mdUrl = url.replace(".html", ".md");
    } else if (url.indexOf(".md") != -1) {
        mdUrl = url;
    } else {
        console.error(`unknown md for url ${url}`);
    }
    return mdUrl;
}

export function insertRenderedFile(config, mdUrl, cb) {
    // TODO handle async failure
    require(["js/marked/lib/marked"], function(marked) {
        let req = new XMLHttpRequest();
        // TODO handle failure
        req.addEventListener("load", listener);
        req.open("GET", mdUrl);
        req.send();

        function listener() {
            let mdText = this.responseText;

            let renderedHtml = marked(mdText);

            config.contentElement.innerHTML = renderedHtml;

            rewriteUrls(config.contentElement);

            cb();
        }
    });
}

export function rewriteUrls(element) {
    let elements = element.getElementsByTagName("a");
    for (let a of elements) {
        if (!a.hasAttribute("href")) {
            continue;
        }

        let oldUrl = a.getAttribute("href");

        let newUrl = htmlUrlFromMdUrl(oldUrl);
        if (newUrl !== oldUrl) {
            console.log(`url rewritten: ${oldUrl}, ${newUrl}`);
        }

        a.setAttribute("href", newUrl);
    }
}

function htmlUrlFromMdUrl(url) {

    url = offsetFromBaseUrl(url);
    
    let ext = ".md";
    let last = url.lastIndexOf(".md");
    if (last === -1) {
        return url;
    }
    let rem = last + ext.length;
    let newUrl = url.substring(0, last) + ".html" + url.substring(rem);
    return newUrl;
}

function offsetFromBaseUrl(url) {
    let urlDir = document.URI_DIR;
    let baseUrl = document.BASE_URI;

    if (!urlDir.startsWith(baseUrl)) {
        throw `URL ${urlDir} is not prefixed by base URL ${baseUrl}`;
    }

    if (isRelative(url)) {
        let prefix = urlDir.substring(baseUrl.length);
        if (prefix.endsWith("/")) {
            prefix = prefix.substring(0, prefix.length - 1);
        }

        if (prefix !== "") {
            url = `${prefix}/${url}`;
        }
    }

    return url;
}

function isRelative(url) {
    if (url.indexOf("://") > -1) {
        return false;
    }

    if (url.startsWith("/")) {
        return false;
    }

    return true;
}

export function showPage() {
    let html = document.getElementsByTagName("html")[0];
    html.style = "display:block;";
}
