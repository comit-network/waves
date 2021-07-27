import * as assert from "assert";
import Debug from "debug";
import { getElementById, setupBrowserWithExtension, setupTestingWallet, switchToWindow, unlockWallet } from "./utils";

describe("e2e tests", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;

    beforeAll(async () => {
        const debug = Debug("e2e-setup");

        let { driver: d, extensionId: eId, extensionTitle: eTitle, webAppTitle: wTitle } =
            await setupBrowserWithExtension(webAppUrl);
        driver = d;
        extensionId = eId;
        extensionTitle = eTitle;
        webAppTitle = wTitle;
    }, 20000);

    afterAll(async () => {
        await driver.quit();
    });

    test("unlock wallet", async () => {
        const debug = Debug("e2e-unlock-wallet");

        debug("Setting up testing wallet");
        await setupTestingWallet(driver, extensionTitle);

        debug("Unlocking wallet");
        await unlockWallet(driver, extensionTitle);

        await driver.navigate().refresh();
        debug("Getting wallet address");
        let addressField = await getElementById(driver, "//p[@data-cy='data-cy-wallet-address-text-field']");
        let address = await addressField.getText();
        debug(`Address found: ${address}`);
        assert(
            "el1qqvq7q42zu99ky2g7n3hmh0yfr8eru0sxk6tutl3hlv240rd8rxyqrsukpsvsqdzc84dvmv6atmzp3f3hassdgmyx5cafy30dp"
                === address,
        );
    }, 40000);
});
