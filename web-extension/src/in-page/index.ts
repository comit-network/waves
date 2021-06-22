const initialize_provider = () => {
    console.log("I was injected 🥳");
    // @ts-ignore
    globalThis.waves = "it works";
};

initialize_provider();
export default {};
