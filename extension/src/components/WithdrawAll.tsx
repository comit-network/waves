import { Button, FormControl, FormErrorMessage, HStack, Input, Text, VStack } from "@chakra-ui/react";
import * as React from "react";
import { ChangeEvent } from "react";
import { useAsync } from "react-async";
import { backgroundPage } from "../background/api";

export default function WithdrawAll() {
    const [withdrawAddress, setWithdrawAddress] = React.useState("");
    const handleWithdrawAddress = (event: ChangeEvent<HTMLInputElement>) => setWithdrawAddress(event.target.value);

    let { isLoading: isWithdrawing, isRejected: withdrawFailed, run: withdraw } = useAsync({
        deferFn: async ([address]) => {
            let page = await backgroundPage();
            return page.withdrawAll(address);
        },
        onReject: (error) => console.log("failed to withdraw funds: %s", error),
    });

    return (<VStack bg="gray.100" align="center" borderRadius={"md"} p={1}>
        <form
            onSubmit={e => {
                e.preventDefault();
                withdraw(withdrawAddress);
            }}
        >
            <Text textStyle="actionable">Withdraw:</Text>
            <HStack>
                <FormControl isInvalid={withdrawFailed}>
                    <Input
                        placeholder="Address"
                        size="md"
                        bg={"white"}
                        value={withdrawAddress}
                        onChange={handleWithdrawAddress}
                    />
                    <FormErrorMessage>Failed to withdraw funds.</FormErrorMessage>
                </FormControl>
                <Button
                    type="submit"
                    variant="primary"
                    isLoading={isWithdrawing}
                >
                    Send
                </Button>
            </HStack>
        </form>
    </VStack>);
}
