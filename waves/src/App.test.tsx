import { render, screen } from "@testing-library/react";
import React from "react";
import App from "./App";

test("renders unlock wallet", () => {
    render(<App />);
    const linkElement = screen.getByText(/Unlock Wallet/i);
    expect(linkElement).toBeInTheDocument();
});
