import { SettingsIcon } from "@chakra-ui/icons";
import { Box, Center, ChakraProvider, Flex, Heading, IconButton, Spacer } from "@chakra-ui/react";
import { faBug } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import * as React from "react";
import { browser } from "webextension-polyfill-ts";
import { Status } from "./background/api";
import AddressQr from "./components/AddressQr";
import WalletBalances from "./components/Balances";
import ConfirmLoanWizard from "./components/ConfirmLoanWizard";
import ConfirmSwap from "./components/ConfirmSwap";
import CreateWallet from "./components/CreateWallet";
import OpenLoans from "./components/OpenLoans";
import UnlockWallet from "./components/UnlockWallet";
import WithdrawAll from "./components/WithdrawAll";
import theme from "./theme";
import { useBalances, useLoanToSign, useOpenLoans, useSwapToSign, useWalletStatus } from "./walletHooks";

const App = () => {
    const { data: walletStatus, reload: reloadWalletStatus, error } = useWalletStatus();
    const { data: balanceUpdates, reload: reloadWalletBalances } = useBalances();
    const { data: swapToSign, reload: reloadSwapToSign } = useSwapToSign();
    const { data: loanToSign, reload: reloadLoanToSign } = useLoanToSign();
    const { data: openLoans, reload: reloadOpenLoans } = useOpenLoans();

    const refreshAll = () => {
        reloadWalletBalances();
        reloadWalletStatus();
        reloadSwapToSign();
        reloadLoanToSign();
        reloadOpenLoans();
    };

    // we want to either sign a swap or the loan but not both:
    let signLoan = false;
    if (!swapToSign && loanToSign) {
        signLoan = true;
    }

    return (
        <ChakraProvider theme={theme}>
            <Box h={600} w={400}>
                {walletStatus?.status === Status.Loaded
                    && <>
                        <Flex>
                            <Center p="2">
                                <Heading size="md">Waves Wallet</Heading>
                            </Center>
                            <Spacer />
                            <Box>
                                <IconButton
                                    aria-label="Settings"
                                    icon={<SettingsIcon />}
                                    onClick={() => browser.runtime.openOptionsPage()}
                                />
                            </Box>
                        </Flex>

                        {balanceUpdates && <WalletBalances balanceUpdates={balanceUpdates} />}
                        {!signLoan && !swapToSign && <AddressQr />}
                        {!signLoan && !swapToSign && <WithdrawAll />}
                        {!signLoan && !swapToSign && <OpenLoans openLoans={openLoans} onRepayed={refreshAll} />}

                        {swapToSign && <ConfirmSwap
                            onCancel={refreshAll}
                            onSuccess={refreshAll}
                            trade={swapToSign!}
                        />}
                        {signLoan
                            && <ConfirmLoanWizard
                                onCancel={refreshAll}
                                onSuccess={refreshAll}
                                loanToSign={loanToSign!}
                            />}
                    </>}
                {walletStatus?.status === Status.NotLoaded
                    && <>
                        <Heading>Unlock Wallet</Heading>
                        <UnlockWallet
                            onUnlock={refreshAll}
                        />
                    </>}
                {walletStatus?.status === Status.None
                    && <>
                        <Heading>Create Wallet</Heading>
                        <CreateWallet
                            onUnlock={refreshAll}
                        />
                    </>}
                {!walletStatus && error
                    && <Center>
                        Something is wrong. Can you catch the <FontAwesomeIcon size="7x" icon={faBug} />?
                        {error.toString()}
                    </Center>}
            </Box>
        </ChakraProvider>
    );
};

export default App;
