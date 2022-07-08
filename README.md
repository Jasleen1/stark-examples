# STARK Fibonacci

A standalone implementation of the Fibonacci AIR example from here: https://github.com/novifinancial/winterfell/tree/main/examples/src/fibonacci

Note that the modifications are minor. 

## Details on this example

Recall that the fibonacci program is traditionally defined as follows: `fib(0) = fib(1) = 1, fib(i) = fib(i-1) + fib(i-2) for i > 1`. 

What we want to build here is the following register model computation:

| **register_0** | **register_1** |
| -------------- | -------------- |
|       1        |      1         |
|       2        |      3         |
|       5        |      8         |
|      ...       |     ...        |

that is, the `k`th row contains the `2k`th and `2k+1`th terms of the Fibonacci sequence. 

**WARNING:** This is a toy implementation intended for educational purposes only. It has not been audited and may contain bugs and security flaws. This implementation is NOT ready for production use.

License
-------

This project is [MIT licensed](./LICENSE).