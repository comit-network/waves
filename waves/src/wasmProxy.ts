export async function hello(msg: string) {
    const { hello } = await import("./wallet-lib/pkg");
    return hello(msg);
}
