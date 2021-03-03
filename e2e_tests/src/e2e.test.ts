import * as assert from "assert";
import { Builder, By, until } from "selenium-webdriver";
import { main } from "ts-node/dist/bin";
import fetch from "node-fetch";

const firefox = require("selenium-webdriver/firefox");
const firefoxPath = require("geckodriver").path;

const getElementById = async (driver, xpath, timeout = 2000) => {
    const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

describe("webdriver", () => {
    let driver;
    let extensionId;
    let websiteWindow;
    let extensionWindow;
    beforeAll(async () => {
        let service = new firefox.ServiceBuilder(firefoxPath);

        const options = new firefox.Options().headless();

        driver = new Builder()
            .setFirefoxService(service)
            .forBrowser("firefox")
            .setFirefoxOptions(options)
            .build();

        driver.getProfil;
        // this works :)
        await driver.installAddon("../extension/web-ext-artifacts/waves_wallet-0.0.1.zip", true);

        // this probably works forever unless we change something and then it won't work anymore
        await driver.get("about:debugging#/runtime/this-firefox");
        const extensionElement = await getElementById(
            driver,
            "//span[contains(text(),'waves_wallet')]//"
                + "parent::li/section/dl/div//dt[contains(text(),'Internal UUID')]/following-sibling::dd",
        );
        extensionId = await extensionElement.getText();

        // Store the ID of the original window
        websiteWindow = await driver.getWindowHandle();

        // Check we don't have other windows open already
        assert((await driver.getAllWindowHandles()).length === 1);

        // Open extension page in new tab
        let extensionUrl = `moz-extension://${extensionId}/popup.html?`;
        await driver.executeScript(`window.open("${extensionUrl}");`);

        // Wait for the new window or tab
        await driver.wait(
            async () => (await driver.getAllWindowHandles()).length === 2,
            10000,
        );

        const windows = await driver.getAllWindowHandles();
        extensionWindow = windows.find((w) => w != websiteWindow);

        await driver.switchTo().window(extensionWindow);

        // Assert that extension window is loaded
        await driver.wait(until.titleIs("Waves Wallet"), 10000);
    });

    afterAll(async () => {
        await driver.quit();
    });

    test("sell swap", async () => {
      await driver.switchTo().window(websiteWindow);
      await driver.get("localhost:3004");
      await driver.wait(until.titleIs("Waves"), 10000);

      // Create wallet
      await driver.switchTo().window(extensionWindow);
      console.log("can switch once");

      await driver.switchTo().window(websiteWindow);
      console.log("can switch twice");

      await driver.switchTo().window(extensionWindow);
      console.log("can switch thrice");

      let password = "foo";
      let passwordInput = await getElementById(driver, "//input[@data-cy=\"create-wallet-password-input\"]");
      await passwordInput.sendKeys(password);

      let createWalletButton = await getElementById(driver, "//button[@data-cy=\"create-wallet-button\"]");
      await createWalletButton.click();

      let addressField = await getElementById(driver, "//p[@data-cy=\"wallet-address-text-field\"]");
      let address = addressField.text;

      // TODO: Do not hard-code website URL
      await fetch(`http://localhost:3004/api/faucet/${address}`, {
        method: "POST",
      });

      await driver.switchTo().window(websiteWindow);
      let alphaAmountInput = await getElementById(driver, "//input[@data-cy=\"Alpha-amount-input\"]");
      await alphaAmountInput.clear();
      await alphaAmountInput.sendKeys("0.4");

      let swapButton = await getElementById(driver, "//button[@data-cy=\"swap-button\"]");
      while (!(await swapButton.isEnabled())) {
        await sleep(2000)
      }

      await swapButton.click();

      // await driver.switchTo().window(extensionWindow);

      // await getElementById(driver, "//button[@data-cy=\"swap-button\"]");
    });
});

export async function sleep(time: number) {
    return new Promise((res) => {
        setTimeout(res, time);
    });
}
