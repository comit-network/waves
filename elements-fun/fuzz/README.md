# Fuzz tests

These fuzz tests make sure that our deserialization code doesn't panic and always exits gracefully.

## Run

To run the fuzzer, simply run the `./fuzz.sh` shell script.

## Debug

If the fuzzer finds input that makes the program crash, look in the crash report for `FUZZ_NAME`.
It will look something like this: `hfuzz_workspace/deserialize_output/SIGABRT.PC.7ffff7da5615.STACK.f512e116a.CODE.-6.ADDR.0.INSTR.mov____0x108(%rsp),%rax.fuzz`

To debug the crash, run the following command (obviously adjusted for the fuzz target and the correct crash file):

```sh
HFUZZ_BUILD_ARGS="--features honggfuzz_fuzz" cargo hfuzz run-debug deserialize_output 'hfuzz_workspace/deserialize_output/SIGABRT.PC.7ffff7da5615.STACK.f512e116a.CODE.-6.ADDR.0.INSTR.mov____0x108(%rsp),%rax.fuzz'
```
