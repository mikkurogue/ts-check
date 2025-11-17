# TS Check

Prettify errors from tsc type checking with very basic suggestions 

The idea behind this project is to create a "in between" layer for the LSP and the client where we can hook in and replace diagnostics in the buffer, to a more readable and most importantly actionable error.

For now its just cheating by calling `tsc --noEmit....` and reading the stdout and stderr from the typechecker itself. I don't think it makes sense to write a custom typechecker for this, so we will just have to be at the mercy of the typescript compiler

Example output;
<img width="1116" height="572" alt="image" src="https://github.com/user-attachments/assets/69c373ac-86bd-41ab-9424-2c478b68995f" />



Insipred by the GOAT [Dillon Mulroy](https://github.com/dmmulroy), where he made a nicer tsc reported neovim plugin.
