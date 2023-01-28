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

## Variables
Variables are supported, and they are calculated during their initialization. Let's say 
there is a variable called `x` and it stores the value $\frac{1}{3}$. Creating variable
`y = x` will result in copying the value of `x` into `y`, so later changes to `x`
will not apply to `y`.

## Operations
* **Addition** - both `Scalars` and `Matrices` support addition operation.
  * For `Scalars` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$, and works as expected for rational numbers.
  * For `Matrices` it is defined as $\hat{Q}^{N\times M} \times \hat{Q}^{N\times M} \to \hat{Q}^{N\times M}$ and works element-wise.
* **Subtraction** - both `Scalars` and `Matrices` support subtraction operation.
  * For both `Scalars` and `Matrices` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$ and works like addition.
* **Multiplication** - both `Scalars` and `Matrices` support multiplication operation.
  * For `Scalars` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$, and works as expected for rational numbers.
  * For `Matrices` it is defined as $\hat{Q}^{N\times M} \times \hat{Q}^{M\times K} \to \hat{Q}^{N\times K}$ and works as expected for matrices.
* **Division** - only `Scalars` support division operation.
  * For `Scalars` it is defined as $\hat{Q} \times \hat{Q} \to \hat{Q}$, and works as expected for rational numbers.
* **Inverse** - only `Matrices` support inverse operation.
  * For `Matrices` it is defined as $\hat{Q}^{N\times N} \to \hat{Q}^{N\times N}$. Inverse $A^{-1}$ of matrix $A$ is defined as $A^{-1}A = AA^{-1} = I$, where $I$ is identity matrix.
* **Echelon** - only `Matrices` support echelon operation.
  * For `Matrices` it is defined as $\hat{Q}^{N\times M} \to \hat{Q}^{N\times M}$. Echelon form is defined [here](https://en.wikipedia.org/wiki/Row_echelon_form).
