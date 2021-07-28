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
        expect(
            "el1qqf09atu8jsmd25eclm8t5relx9q3x80yw3yncesd3ez2rgzy2pxkmamlcgrwlfa523rd4vjmcg4anyv2w89kk74k5p0af0pj5").toBe(
                address
        );
    }, 40000);
});
