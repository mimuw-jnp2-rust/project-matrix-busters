# Usage

## Types

### Scalar

Scalars are 2x64-bit rational numbers - let's call this field $\hat{Q}$. They can be signed. Examples

```matlab
10
-45
2/13
-75/3
```

Rational numbers are always normalized e.g. if number is in form $\frac{p}{q}$ it will be
displayed as $\frac{p/g}{q/g}$, where $g=\text{gcd}(p, q)$. Also if $q/g = 1$ the number is displayed as an integer.

Scalars can be both added in *shell* or via *GUI editor*.

### Matrix

Matrices are 2D arrays of Scalars. More precisely matrix $A \in \hat{Q}^{N\times M}$
is a matrix over field $\hat{Q}$ with width $N$ and height $M$.

Currently, the only way to create a matrix is to use *GUI editor*.

## Warning

A matrix $M\in \hat{Q}^{1,1}$ is **not** a scalar. It is a matrix with one element.

## Variables

Variables are supported, and they are calculated during their initialization. Let's say
there is a variable called `x` and it stores the value $\frac{1}{3}$. Creating variable
`y = x` will result in copying the value of `x` into `y`, so later changes to `x`
will not apply to `y`.

## Operations

* **Addition** - both `Scalars` and `Matrices` support addition operation.
    * For `Scalars` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$, and works as expected for rational numbers.
    * For `Matrices` it is defined as $\hat{Q}^{N\times M} \times \hat{Q}^{N\times M} \to \hat{Q}^{N\times M}$ and works
      element-wise.
* **Subtraction** - both `Scalars` and `Matrices` support subtraction operation.
    * For both `Scalars` and `Matrices` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$ and works like addition.
* **Multiplication** - both `Scalars` and `Matrices` support multiplication operation.
    * For `Scalars` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$, and works as expected for rational numbers.
    * For `Matrices` it is defined as $\hat{Q}^{N\times M} \times \hat{Q}^{M\times K} \to \hat{Q}^{N\times K}$ and works as expected for matrices.
    * For `Matrices` and `Scalars` it is defined as $\hat{Q}^{N\times M} \times \hat{Q} \to \hat{Q}^{N\times M}$ and works as expected for matrices and scalars.
* **Division** - only `Scalars` support division operation.
    * For `Scalars` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$, and works as expected for rational numbers.
* **Inverse** - only `Matrices` support inverse operation.
    * For `Matrices` it is defined as $\hat{Q}^{N\times N} \to \hat{Q}^{N\times N}$. Inverse $A^{-1}$ of matrix $A$ is
      defined as $A^{-1}A = AA^{-1} = I$, where $I$ is identity matrix.
* **Echelon** - only `Matrices` support echelon operation.
    * For `Matrices` it is defined as $\hat{Q}^{N\times M} \to \hat{Q}^{N\times M}$. Echelon form is
      defined [here](https://en.wikipedia.org/wiki/Row_echelon_form).
* **Power** - both `Scalars` and `Matrices` support power operation.
    * For `Scalars` it is defined as $\hat{Q} \times \mathbb{N} \to \hat{Q}$, and works as expected for rational numbers.
    * For `Matrices` it is defined as $\hat{Q}^{N\times N} \times \mathbb{N} \to \hat{Q}^{N\times N}$. Power $A^k$ of
      matrix $A$ is defined as $A^k = A \cdot A \cdot \dots \cdot A$ where $k$ is a positive integer.

## Examples

```matlab
x = 1/3
y = 2/3
M = [1 2; 5 3]  % This syntax is not supported yet.
N = [6 7; 3 1]  % Use GUI editor instead.
% Addition
z = x + y   % z = 1
P = M + N   % P = [7 9; 8 4]
% Subtraction
z = x - y   % z = -1/3
P = M - N   % P = [-5 -5; 2 2]
% Multiplication
z = x * y   % z = 2/9
P = M * N   % P = [18 10; 36 22]
% Division
z = x / y   % z = 1/2
% Inverse (only in GUI)
P = inv(M)    % P = 1/7 * [-3 2; 5 -1]
% Echelon (only in GUI)
P = echelon(M)    % P = [1 0; 0 1]
% Power
z = x^2   % z = 1/9
P = M^2   % P = [11 8; 20 19]
```

## Shell

Shell is a command line interface for the calculator. It is used to input commands.
Supported commands are:

* `x = <expression>` - creates a variable `x` and assigns it the value of `<expression>`.
* `<expression>` - evaluates `<expression>` and stores it in special variable `$`.
  Error messages are displayed as a popup notification toast.

These are the rules expressed in BNF:
```bnf
<digit>      ::= "0" | "1" | ... | "9"
<integer>    ::= <digit>+
<letter>     ::= "a" | "ą" | "b" | ... | "ż"
<identifier> ::= (<letter> | "_") (<letter> | <digit> | "_")* | "$"
<unary_op>   ::= "+" | "-"
<binary_op>  ::= "+" | "-" | "*" | "/"
<expr>       ::= <integer> | <identifier> | <expr> <binary_op> <expr> | "(" <expr> ")" | <unary_op> <expr>
```

### Examples
```matlab
v = 1/3 + 4/15 - 4/19 * 2/3 - 4^5 * (3/4 - 2/3)
w = ((((4/3 + 5/2) * 14) - 44) / 2) ^ 2
N = M^14 - Z * 4 * (M - Z)    % where M, Z are square matrices
a = -v
very_simple_NAME_123 = 1/3
```

## GUI

GUI is a graphical user interface for the calculator. All objects created in current environment are displayed
on `Objects` list. Clicking on an object will open a new window with object's properties. In such window,
the value can be edited. If the value is edited, the object will be updated.
There are certain operations that can be performed on objects:

* `Scalar`
    * `Inverse` - calculates inverse of the scalar, copies its LaTeX representation to clipboard and stores the
      numerical value in `$`.
    * `LaTeX` - copies the scalar's LaTeX representation to clipboard.
* `Matrix`
    * `Inverse` - calculates inverse of the matrix, copies its LaTeX representation to clipboard and stores the
      numerical value in `$`.
    * `LaTeX` - copies the matrix's LaTeX representation to clipboard.
      If an error occurs during the operation, the error message will be displayed as a popup toast.
    * `Echelon` - calculates echelon form of the matrix, stores the numerical value in `$` and copies *all* transitions
      in LaTeX to clipboard.

### Echelon LaTeX example

Let's say we have a matrix

```math
A = \begin{bmatrix} 1 & 2 & 3 \\ 11 & 67 & 2 \\ 8 & 1 & 34 \end{bmatrix}.
```

`Echelon` operation will result in LaTeX code representing this:

```math
\left[\begin{array}{ccc}
1 & 2 & 3\\11 & 67 & 2\\8 & 1 & 34
\end{array}\right]
\xrightarrow{\substack{w_{2} - 11w_{1}\\w_{3} - 8w_{1}}} \left[\begin{array}{ccc}
1 & 2 & 3\\0 & 45 & -31\\0 & -15 & 10
\end{array}\right]
\xrightarrow{w_{2} : 45} \left[\begin{array}{ccc}
1 & 2 & 3\\0 & 1 & -\frac{31}{45}\\0 & -15 & 10
\end{array}\right]
\xrightarrow{\substack{w_{1} - 2w_{2}\\w_{3} + 15w_{2}}} \left[\begin{array}{ccc}
1 & 0 & \frac{197}{45}\\0 & 1 & -\frac{31}{45}\\0 & 0 & -\frac{1}{3}
\end{array}\right]
\xrightarrow{w_{3} : \left(-\frac{1}{3}\right)} \left[\begin{array}{ccc}
1 & 0 & \frac{197}{45}\\0 & 1 & -\frac{31}{45}\\0 & 0 & 1
\end{array}\right]
\xrightarrow{\substack{w_{1} - \frac{197}{45}w_{3}\\w_{2} + \frac{31}{45}w_{3}}} \left[\begin{array}{ccc}
1 & 0 & 0\\0 & 1 & 0\\0 & 0 & 1
\end{array}\right]
```

This may not look useful, as it produced an identity matrix, but when we take a different matrix

```math
B = \left[\begin{array}{cccc}1 & 4 & 0 & 15\\6 & 11 & 8 & 4\\-1 & 3 & 6 & -6\end{array}\right]
```

which represents a system of linear equations, we get a much more useful result:

```math
\left[\begin{array}{cccc}1 & 4 & 0 & 15\\6 & 11 & 8 & 4\\-1 & 3 & 6 & -6\end{array}\right]
\xrightarrow{\substack{w_{2} - 6w_{1}\\w_{3} + w_{1}}} \left[\begin{array}{cccc}1 & 4 & 0 & 15\\0 & -13 & 8 & -86\\0 & 7 & 6 & 9\end{array}\right]
\xrightarrow{w_{2} : \left(-13\right)} \left[\begin{array}{cccc}1 & 4 & 0 & 15\\0 & 1 & -\frac{8}{13} & \frac{86}{13}\\0 & 7 & 6 & 9\end{array}\right]
\xrightarrow{\substack{w_{1} - 4w_{2}\\w_{3} - 7w_{2}}} \left[\begin{array}{cccc}1 & 0 & \frac{32}{13} & -\frac{149}{13}\\0 & 1 & -\frac{8}{13} & \frac{86}{13}\\0 & 0 & \frac{134}{13} & -\frac{485}{13}\end{array}\right]
\xrightarrow{w_{3} : \frac{134}{13}} \left[\begin{array}{cccc}1 & 0 & \frac{32}{13} & -\frac{149}{13}\\0 & 1 & -\frac{8}{13} & \frac{86}{13}\\0 & 0 & 1 & -\frac{485}{134}\end{array}\right]
\xrightarrow{\substack{w_{1} - \frac{32}{13}w_{3}\\w_{2} + \frac{8}{13}w_{3}}} \left[\begin{array}{cccc}1 & 0 & 0 & -\frac{171}{67}\\0 & 1 & 0 & \frac{294}{67}\\0 & 0 & 1 & -\frac{485}{134}\end{array}\right]
```

Producing a row echelon form is a very tedious task, but with this calculator, it is as easy as clicking a button.
It may be very useful for students, as it can be used to create LaTeX for their homework.

Another useful application is to calculate the inverse of a matrix. Let's say we have a matrix
```math
Z = \left[\begin{array}{cc}1 & 2\\6 & 1\end{array}\right]
```
If we produce a matrix
```math
Z' = \left[\begin{array}{cc|cc}1 & 2 & 1 & 0\\6 & 1 & 0 & 1\end{array}\right]
```
and perform `Echelon` operation on it, we get
```math
\left[\begin{array}{cccc}1 & 2 & 1 & 0\\6 & 1 & 0 & 1\end{array}\right]
\xrightarrow{\substack{w_{2} - 6w_{1}}} \left[\begin{array}{cccc}1 & 2 & 1 & 0\\0 & -11 & -6 & 1\end{array}\right]
\xrightarrow{w_{2} : \left(-11\right)} \left[\begin{array}{cccc}1 & 2 & 1 & 0\\0 & 1 & \frac{6}{11} & -\frac{1}{11}\end{array}\right]
\xrightarrow{\substack{w_{1} - 2w_{2}}} \left[\begin{array}{cccc}1 & 0 & -\frac{1}{11} & \frac{2}{11}\\0 & 1 & \frac{6}{11} & -\frac{1}{11}\end{array}\right]
```
and as surprising as it may seem, the inverse of `Z` is
```math
\left[\begin{array}{cc}-\frac{1}{11} & \frac{2}{11}\\\frac{6}{11} & -\frac{1}{11}\end{array}\right]
```
We just inverted the matrix `Z` by performing elementary row operations on the augmented matrix and got all transformations in LaTeX. 
Finally, the `Inverse` operation will also produce LaTeX code representing all the transformations.


## GUI editor

GUI editor is a graphical interface for creating matrices and scalars. To open it click on `Add matrix` or `Add scalar`
button.
A new variable has to have a name and a value, that can be evaluated using existing environment variables.
If provided value is invalid, an error message will be displayed and new variable will not be created.

## Features
If you get bored with plain background and want to spice things up, you can turn `fft` feature on.
It will draw an image of a Fourier transformed image provided in `assets/`. The other way to change
the background is to turn `clock` feature on. It will draw a fractal clock in the background. If both 
`fft` and `clock` are turned on, `fft` will prioritize `clock` - only if image file is missing
the clock will be drawn.
