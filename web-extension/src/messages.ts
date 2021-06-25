export enum MessageKind {
    WalletStatusRequest = "WalletStatusRequest",
    WalletStatusResponse = "WalletStatusResponse",
    SellRequest = "SellRequest",
    SellResponse = "SellResponse",
    BuyRequest = "BuyRequest",
    BuyResponse = "BuyResponse",
    LoanRequest = "LoanRequest",
    LoanResponse = "LoanResponse",
    AddressRequest = "AddressRequest",
    AddressResponse = "AddressResponse",
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
