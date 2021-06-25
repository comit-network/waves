export enum MessageKind {
    WalletStatusRequest = "WalletStatusRequest",
    WalletStatusResponse = "WalletStatusResponse",
    SellRequest = "SellRequest",
    SellResponse = "SellResponse",
}

export enum Direction {
    ToBackground = "ToBackground",
    ToPage = "ToPage",
}

export interface Message<T> {
    kind: MessageKind;
    direction: Direction;
    payload: T;
}
