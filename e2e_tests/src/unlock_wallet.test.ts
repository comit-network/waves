import Debug from "debug";
import { WebDriver } from "selenium-webdriver";
import { getElementById, setupBrowserWithExtension, switchToWindow, unlockWallet } from "./utils";

describe("unlock wallet test", () => {
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
            "el1qqf09atu8jsmd25eclm8t5relx9q3x80yw3yncesd3ez2rgzy2pxkmamlcgrwlfa523rd4vjmcg4anyv2w89kk74k5p0af0pj5",
        ).toBe(
            address,
        );
    }, 40000);
});

// set testing wallet.
// seed words are: `globe favorite camp draw action kid soul junk space soda genre vague name brisk female circle equal fix decade gloom elbow address genius noodle`
// wallet password is: `foo`.
const setupTestingWallet = async (driver: WebDriver, extensionTitle: string) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$brb7JqhPEJbIxXOu/Jn3Aw==$8eWWdvAarl6IuOjViqAZuqhq05aD/YBjTAtioqtoC9U=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','4b83ff6937ce2650354d02d9c7f6d9b828824c1dbe4d0795b3b14d9c9042dbca$0c130f39ada0ec0e9e0e2a2af91815f7e77f8ed64481467f19e42bcdfeaf88af2d10df9e9143223b71e053f67fad830ec102c7e977bd646cd1dec3a2718c97dc7e98cd349c7a3c2147f78c9b8b6436f68b96e2bf73f11ca496483175ad5dfa577d1efb40827455b05e3c51f9745ed8d4726d49f4a54470073efea876815f25');",
    );

    return "el1qqf09atu8jsmd25eclm8t5relx9q3x80yw3yncesd3ez2rgzy2pxkmamlcgrwlfa523rd4vjmcg4anyv2w89kk74k5p0af0pj5";
};
