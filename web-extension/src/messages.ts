export enum MessageKind {
    WalletStatusRequest = "WalletStatusRequest",
    WalletStatusResponse = "WalletStatusResponse",
    SellRequest = "SellRequest",
    SellResponse = "SellResponse",
    BuyRequest = "BuyRequest",
    BuyResponse = "BuyResponse",
    LoanRequest = "LoanRequest",
    LoanResponse = "LoanResponse", // TODO: Choose a better name
    SignAndSendSwap = "SignAndSendSwap",
    SwapTxid = "SwapTxid",
    AddressRequest = "AddressRequest",
    AddressResponse = "AddressResponse",
    SignLoan = "SignLoan",
    SignedLoan = "SignedLoan",
    LoanRejected = "LoanRejected",
    SwapRejected = "SwapRejected",
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
