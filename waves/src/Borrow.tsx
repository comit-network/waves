import {
    Box,
    Button,
    Center,
    Divider,
    Text,
    Tooltip,
    useToast,
    HStack,
    VStack,
    useRadioGroup
} from "@chakra-ui/react";
import Debug from "debug";
import React, { Dispatch } from "react";
import { AsyncState, useAsync } from "react-async";
import { useHistory } from "react-router-dom";
import { Action, BorrowState, Rate } from "./App";
import {Collateralization, getLoanOffer, postLoanFinalization, postLoanRequest, Term} from "./Bobtimus";
import NumberInput from "./components/NumberInput";
import RateInfo from "./components/RateInfo";
import WavesProvider from "./waves-provider";
import { Status, WalletStatus } from "./waves-provider/wavesProvider";
import {RadioCard} from "./RadioButton";

const debug = Debug("Borrow");
const error = Debug("Borrow:error");

interface BorrowProps {
    dispatch: Dispatch<Action>;
    rate: Rate;
    state: BorrowState;
    walletStatusAsyncState: AsyncState<WalletStatus>;
    wavesProvider: WavesProvider | undefined;
}

interface TermRadioGroupProps {
    dispatch: Dispatch<Action>;
    terms: Term[]
}

function TermRadioGroup({dispatch, terms}: TermRadioGroupProps) {
    const { getRootProps, getRadioProps } = useRadioGroup({
        name: "framework",
        defaultValue: "react",
        onChange: (data) => {
            debug("term selected " + data);
            dispatch({
                type: "UpdateLoanTerm",
                value:  Number.parseFloat(data.toString()),
            });
        },
    });

    const group = getRootProps();

    return (
        <HStack {...group}>
            {terms.map((value) => {
                const radio = getRadioProps({ value: value.days.toString() })
                return (
                    <RadioCard key={value.days.toString()} {...radio}>
                        {value.days}
                    </RadioCard>
                )
            })}
        </HStack>
    );
}

interface CollateralizationRadioGroupProps {
    dispatch: Dispatch<Action>;
    collateralizations: Collateralization[]
}

function CollateralizatinRadioGroup({dispatch, collateralizations}: CollateralizationRadioGroupProps) {
    const { getRootProps, getRadioProps } = useRadioGroup({
        name: "framework",
        defaultValue: "react",
        onChange: (data) => {
            debug("collateralization selected " + data);
            dispatch({
                type: "UpdateLoanCollateralization",
                value:  Number.parseFloat(data.toString()),
            });
        },
    });

    const group = getRootProps();

    return (
        <HStack {...group}>
            {collateralizations.map((value) => {
                const radio = getRadioProps({ value: value.collateralization.toString() })
                return (
                    <RadioCard key={value.collateralization.toString()} {...radio}>
                        {(value.collateralization * 100) + "%"}
                    </RadioCard>
                )
            })}
        </HStack>
    );
}

function Borrow({ dispatch, state, rate, wavesProvider, walletStatusAsyncState }: BorrowProps) {
    const toast = useToast();
    const history = useHistory();

    const loanOfferHook = useAsync({
        promiseFn: getLoanOffer,
        onResolve: (data) => {
            dispatch({
                type: "UpdateLoanOffer",
                value: data,
            });
        },
    });
    let { isLoading: loanOfferLoading, data: loanOffer } = loanOfferHook;
    // TODO: Some mechanism to refresh the loan offer values
    //  Note: The rate is currently not taken from the offer, but from the state's rate

    let { data: walletStatus, reload: reloadWalletStatus, error: walletStatusError } = walletStatusAsyncState;

    function onPrincipalAmountChange(newAmount: string) {
        dispatch({
            type: "UpdatePrincipalAmount",
            value: newAmount,
        });
    }

    const interestRate = loanOffer ? loanOffer.base_interest_rate : 0;
    const minPrincipal = loanOffer ? loanOffer.min_principal : 0;
    const maxPrincipal = loanOffer ? loanOffer.max_principal : 0;

    const principalAmount = Number.parseFloat(state.principalAmount);

    // TODO: Let the user define the collateral amount that is within Bobtimus' LTV bounds
    let interestAmount = principalAmount * interestRate;
    let repaymentAmount = principalAmount + interestAmount;

    // The bid price is used so the lender is covered under the assumption of selling the asset
    let collateralAmount = (repaymentAmount * state.collateralization) / rate.bid;

    let terms = loanOffer ? loanOffer.terms : [];
    let collateralizations = loanOffer ? loanOffer.collateralizations : [];

    let { run: takeLoan, isLoading: isTakingLoan } = useAsync({
        deferFn: async () => {
            if (!wavesProvider) {
                error("Cannot borrow. Waves provider not found.");
                return;
            }

            try {
                const feeRate = state.loanOffer!.fee_sats_per_vbyte;

                let loanRequestWalletParams = await wavesProvider.makeLoanRequestPayload(
                    collateralAmount.toString(),
                    feeRate.toString(),
                );

                let loanResponse = await postLoanRequest(
                    loanRequestWalletParams,
                    state.loanTermInDays,
                    state.collateralization,
                    principalAmount,
                );
                debug(JSON.stringify(loanResponse));

                let loanTransaction = await wavesProvider.signLoan(loanResponse);
                let txid = await postLoanFinalization(loanTransaction);

                // TODO: Add different page for loaned?
                history.push(`/trade/swapped/${txid}`);
            } catch (e) {
                const description = typeof e === "string" ? e : JSON.stringify(e);

                toast({
                    title: "Error",
                    description,
                    status: "error",
                    duration: 5000,
                    isClosable: true,
                });
            }
        },
    });

    async function get_extension() {
        // TODO forward to firefox app store
        debug("Download our awesome extension from...");
        reloadWalletStatus();
    }

    async function unlock_wallet() {
        // TODO send request to open popup to unlock wallet
        debug("For now open popup manually...");
        reloadWalletStatus();
    }

    let loanButton;
    if (!wavesProvider || walletStatusError) {
        if (walletStatusError) {
            error(walletStatusError);
        }
        loanButton = <Button
            onClick={async () => {
                await get_extension();
            }}
            variant="primary"
            w="15rem"
            data-cy="get-extension-button"
        >
            Get Extension
        </Button>;
    } else {
        switch (walletStatus?.status) {
            case Status.None:
            case Status.NotLoaded:
                loanButton = <Button
                    onClick={async () => {
                        await unlock_wallet();
                    }}
                    variant="primary"
                    w="15rem"
                    data-cy="unlock-wallet-button"
                >
                    Unlock Wallet
                </Button>;
                break;
            case Status.Loaded:
                loanButton = <Button
                    variant="primary"
                    w="15rem"
                    isLoading={isTakingLoan}
                    onClick={takeLoan}
                    data-cy="data-cy-take-loan-button"
                >
                    Take loan
                </Button>;
                break;
        }
    }

    return (
        <VStack spacing={4} align="stretch">
            <Center w={800} h={320}>
                <HStack spacing={4}>
                    <VStack padding={4} spacing={4} align="stretch" bg="gray.100" borderRadius={"md"}>
                        <Text align={"left"} fontWeight="bold">I want to borrow</Text>
                        <Tooltip
                            label={"min = " + minPrincipal + " max = " + maxPrincipal}
                            aria-label="principal"
                            hasArrow
                            placement={"right"}
                        >
                            <Box>
                                <NumberInput
                                    currency="$"
                                    value={state.principalAmount}
                                    precision={2}
                                    step={0.01}
                                    onAmountChange={onPrincipalAmountChange}
                                    isDisabled={loanOfferLoading}
                                    dataCy={"data-cy-principal"}
                                />
                            </Box>
                        </Tooltip>
                        <Text align={"left"} fontWeight="bold">For a loan term of</Text>
                        <TermRadioGroup dispatch={dispatch} terms={terms}/>
                        <Text align={"left"} fontWeight="bold">At a collateralization rate of</Text>
                        <CollateralizatinRadioGroup dispatch={dispatch} collateralizations={collateralizations} />
                    </VStack>
                    <Divider orientation={"vertical"} colorscheme="gray.600" />
                    <VStack
                        spacing={4}
                        align="stretch"
                    >
                        <Text align={"left"} fontWeight="bold">Collateral amount to be locked up:</Text>
                        <NumberInput
                            currency="₿"
                            value={collateralAmount}
                            precision={7}
                            step={0.000001}
                            onAmountChange={() => {}}
                            isDisabled={true}
                            dataCy={"data-cy-collateral"}
                            textColor={"black"}
                        />
                        <Text align={"left"} fontWeight="bold">Interest rate for the loan:</Text>
                        <NumberInput
                            currency="%"
                            value={interestRate * 100}
                            precision={4}
                            step={0.01}
                            onAmountChange={() => {}}
                            isDisabled={true}
                            dataCy={"data-cy-interest-rate"}
                            textColor={"black"}
                        />
                        <Text align={"left"} fontWeight="bold">Interest amount paid upon repayment:</Text>
                        <NumberInput
                            currency="$"
                            value={interestAmount}
                            precision={7}
                            step={0.01}
                            onAmountChange={() => {}}
                            isDisabled={true}
                            dataCy={"data-cy-interest"}
                            textColor={"black"}
                        />
                    </VStack>
                </HStack>
            </Center>

            <RateInfo rate={rate} direction={"ask"} />

            <Center>
                {loanButton}
            </Center>
        </VStack>
    );
}

export default Borrow;
