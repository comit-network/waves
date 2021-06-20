import Debug from "debug";

Debug.enable("*");
const debug = Debug("content");

// If your extension doesn't need a content script, just leave this file empty

// This is an example of a script that will run on every page. This can alter pages
// Don't forget to change `matches` in manifest.json if you want to only change specific webpages
printAllPageLinks();

// This needs to be an export due to typescript implementation limitation of needing '--isolatedModules' tsconfig
export function printAllPageLinks() {
    const allLinks = Array.from(document.querySelectorAll("a")).map(
        link => link.href,
    );

    debug("-".repeat(30));
    debug(
        `These are all ${allLinks.length} links on the current page that have been printed by the Sample Create React Extension`,
    );
    debug(allLinks);
    debug("-".repeat(30));
}
