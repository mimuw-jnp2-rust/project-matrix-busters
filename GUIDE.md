# Usage

## Types
### Scalar
Scalars are 2x64-bit rational numbers - $\hat{Q}$. They can be signed. Examples
```matlab
10
-45
2/13
-75/3
```
Rational numbers are always normalized e.g. if number is in form $\frac{p}{q}$ it will be 
displayed as $\frac{p/g}{q/g}$, where $g=\text{gcd}(p, q)$. Also if $q/g = 1$ the number is displayed as an integer.

Scalars can be both added in *shell* or via GUI editor. 

### Matrix
Matrices are 2D arrays of Scalars. More precisely matrix $A \in \hat{Q}^{N\times M}$
is a matrix over field $\hat{Q}$ with width $N$ and height $M$.

## Operations
* **Addition** - both `Scalars` and `Matrices` support addition operation.


