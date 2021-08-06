import { Alert, AlertDescription, AlertIcon, AlertTitle, Button, Flex, VStack } from "@chakra-ui/react";
import { Step, Steps, useSteps } from "chakra-ui-steps";
import * as React from "react";
import { useState } from "react";
import { FiCheck, FiClipboard, FiExternalLink } from "react-icons/all";
import { confirmLoan, createLoanBackup } from "../background-proxy";
import { LoanToSign } from "../models";
import ConfirmLoan from "./ConfirmLoan";

interface ConfirmLoanWizardProps {
    onCancel: () => void;
    onSuccess: () => void;
    loanToSign: LoanToSign;
}

const ConfirmLoanWizard = ({ onCancel, onSuccess, loanToSign }: ConfirmLoanWizardProps) => {
    const { nextStep, activeStep } = useSteps({
        initialStep: 0,
    });
    let [signedTransaction, setSignedTransaction] = useState("");

    const onSigned = (tx: string) => {
        setSignedTransaction(tx);
        nextStep();
    };

    const downloadLoanBackup = async () => {
        const loanBackup = await createLoanBackup(signedTransaction);
        const file = new Blob([JSON.stringify(loanBackup)], { type: "text/json" });
        const url = URL.createObjectURL(file);
        // Note: it would be nicer to open a dialog to download the file. However, we would lose focus
        // of the window. We don't want that, hence we just open a new tab with the content
        window.open(url, "_blank");
        nextStep();
    };

    const onPublish = async () => {
        await confirmLoan(signedTransaction);
        onSuccess();
    };

    return (
        <VStack width="100%">
            <Steps activeStep={activeStep}>
                <Step label={"Sign"} key={"sign"} icon={FiClipboard}>
                    <Flex py={4}>
                        <ConfirmLoan
                            loanToSign={loanToSign}
                            onCancel={onCancel}
                            onSigned={onSigned}
                        />
                    </Flex>
                </Step>
                <Step label={"Backup"} key={"backup"} icon={FiExternalLink}>
                    <Flex py={4}>
                        <VStack>
                            <Alert
                                status="warning"
                                variant="subtle"
                                flexDirection="column"
                                alignItems="center"
                                justifyContent="center"
                                textAlign="center"
                                height="250px"
                            >
                                <AlertIcon boxSize="40px" mr={0} />
                                <AlertTitle mt={4} mb={1} fontSize="lg">
                                    Create a backup!
                                </AlertTitle>
                                <AlertDescription>
                                    Click below to download a backup of the loan details. Together with your seed words
                                    you can recover your loan details in case your browser storage gets purged. Keep it
                                    safe! You can restore your backup through the settings.
                                </AlertDescription>
                            </Alert>
                            <Button
                                onClick={async () => {
                                    await downloadLoanBackup();
                                }}
                                data-cy="data-cy-download-loan-button"
                            >
                                Download backup
                            </Button>
                        </VStack>
                    </Flex>
                </Step>

                <Step label={"Confirm"} key={"confirm"} icon={FiCheck}>
                    <Flex py={4}>
                        <VStack>
                            <Alert
                                status="info"
                                variant="subtle"
                                flexDirection="column"
                                alignItems="center"
                                justifyContent="center"
                                textAlign="center"
                                height="200px"
                            >
                                <AlertIcon boxSize="40px" mr={0} />
                                <AlertTitle mt={4} mb={1} fontSize="lg">
                                    Backup saved?
                                </AlertTitle>
                                <AlertDescription>
                                    The lender will publish the transaction once signed.
                                </AlertDescription>
                            </Alert>
                            <Button
                                onClick={() => onPublish()}
                                data-cy="data-cy-confirm-loan-button"
                            >
                                Confirm
                            </Button>
                        </VStack>
                    </Flex>
                </Step>
            </Steps>
        </VStack>
    );
};

export default ConfirmLoanWizard;
