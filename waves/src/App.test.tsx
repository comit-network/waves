import { render, screen } from "@testing-library/react";
import React from "react";
import App from "./App";

test("renders hello world", () => {
    render(<App />);
    const linkElement = screen.getByText(/Rust lib says:/i);
    expect(linkElement).toBeInTheDocument();
});
