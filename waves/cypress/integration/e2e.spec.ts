/// <reference types="Cypress" />

describe("The app", () => {
    it("should let you sell bitcoin", () => {
        cy.visit("/");

        // just some assertions
        cy.contains("Swap");
        cy.contains("Create wallet");

        // create wallet procedure
        cy.get("[data-cy=create-wallet-button]").click();
        cy.get("[data-cy=wallet-password-input]").type("foo");
        cy.get("[data-cy=create-wallet-form]").submit();

        // open wallet to get address
        cy.get("[data-cy=wallet-info-button]").click();
        cy.get("[data-cy=wallet-address-textfield]").invoke("text").then((address) => {
            cy.request("POST", `/api/faucet/${address}`);
        });
        cy.get("[data-cy=wallet-address-textfield]").type("{esc}");

        // swap
        cy.get("[data-cy=Alpha-amount-input]").find("input").clear();
        cy.get("[data-cy=Alpha-amount-input]").find("input").type("0.42");

        // Sign with wallet
        // TODO verify all numbers
        cy.get("[data-cy=swap-button]", { timeout: 20 * 1000 }).should("be.enabled");
        cy.get("[data-cy=swap-button]").click();

        cy.get("[data-cy=sign-and-send-button]").click();

        cy.url().should("include", "/swapped/");
    });
    it("should let you buy bitcoin", () => {
        cy.visit("/");

        // create wallet procedure
        cy.get("[data-cy=create-wallet-button]").click();
        cy.get("[data-cy=wallet-password-input]").type("foo");
        cy.get("[data-cy=create-wallet-form]").submit();

        // open wallet to get address
        cy.get("[data-cy=wallet-info-button]").click();
        cy.get("[data-cy=wallet-address-textfield]").invoke("text").then((address) => {
            cy.request("POST", `/api/faucet/${address}`);
        });
        cy.get("[data-cy=wallet-address-textfield]").type("{esc}");

        // swap
        cy.get("[data-cy=exchange-asset-types-button]").click();
        cy.get("[data-cy=Alpha-amount-input]").find("input").clear();
        cy.get("[data-cy=Alpha-amount-input]").find("input").type("10000.0");

        // Sign with wallet
        cy.get("[data-cy=swap-button]", { timeout: 20 * 1000 }).should("be.enabled");
        cy.get("[data-cy=swap-button]").click();

        cy.get("[data-cy=sign-and-send-button]").click();

        cy.url().should("include", "/swapped/");
    });
});
