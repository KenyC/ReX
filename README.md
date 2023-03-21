# ReX &nbsp; [![](https://tokei.rs/b1/github/cbreeden/rex)](https://github.com/KenyC/ReX)

<p align="center"><img src="rex.png" alt="ReX logo" width="300px"/></p>
<h3 align="center">Typesetting Mathematics</h3>

This is a fork of [ReX](https://github.com/cbreeden/rex), a Rust mathematical typesetting engine.

# Why the fork?

This fork of ReX is designed to allow users to use their preferred rendering engine and font parser, instead of relying on Pathfinder and the font-parsing crate `font` (whose continuing development is not guaranteed). Implementing the trait MathFont allows users to specify their own font parser, and implementing `Backend<F : MathFont>`, where F is the type of the desired font parser, allows users to use their own rendering engine. 

The following features define some implementations of these traits for you:

  - `fontcrate-backend`: uses the `font` crate for font parsing
  - `ttfparser-backend`: uses the `ttf-parser` crate for font parsing
  - `pathfinder-backend`: in combination with `fontcrate-backend`, implements `Backend<OpenTypeFont>` the `pathfinder`for rendering 
  - `femtovg-backend`: uses the `femtovg` for rendering ; in combination with `fontcrate-backend`, implements `Backend<OpenTypeFont>` ; in combination with `ttfparser-backend`, implements `Backend<ttf_parser::Font>`.


Perhaps, this fork may ultimately turn into an attempt to take full ownership of the engine.

# Samples

Note: ReX rendered all of these examples in SVG, but due to limitations in SVG rendering on GitHub, we need to convert them to PNG.
See the `samples/` folder for the original SVG source.

### The Quadratic Fromula
`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`

![Example](samples/The_Quadratic_Fromula.png)

### Double angle formula for Sine
`\sin(\theta + \phi) = \sin(\theta)\cos(\phi) + \sin(\phi)\cos(\theta)`

![Example](samples/Double_angle_formula_for_Sine.png)

### Divergence Theorem
`\int_D (\nabla \cdot F)\,\mathrm{d}V = \int_{\partial D} F \cdot n\,\mathrm{d}S`

![Example](samples/Divergence_Theorem.png)

### Standard Deviation
`\sigma = \sqrt{ \frac{1}{N} \sum_{i=1}^N (x_i - \mu)^2 }`

![Example](samples/Standard_Deviation.png)

### Fourier Inverse
`f(x) = \int_{-\infty}^{\infty} \hat f(\xi) e^{2\pi i \xi x}\,\mathrm{d}\xi`

![Example](samples/Fourier_Inverse.png)

### Cauchy-Schwarz Inequality
`\left\vert \sum_k a_kb_k \right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}`

![Example](samples/Cauchy-Schwarz_Inequality.png)

### Exponent
`e = \lim_{n \to \infty} \left(1 + \frac{1}{n}\right)^n`

![Example](samples/Exponent.png)

### Ramanujan's Identity
`\frac{1}{\pi} = \frac{2\sqrt{2}}{9801} \sum_{k=0}^\infty \frac{ (4k)! (1103+26390k) }{ (k!)^4 396^{4k} }`

![Example](samples/Ramanujan's_Identity.png)

### A surprising identity
`\int_{-\infty}^{\infty} \frac{\sin(x)}{x}\,\mathrm{d}x = \int_{-\infty}^{\infty}\frac{\sin^2(x)}{x^2}\,\mathrm{d}x`

![Example](samples/A_surprising_identity.png)

### Another gem from Ramanujan
`\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \cdots}}}}`

![Example](samples/Another_gem_from_Ramanujan.png)

### Another gem from Cauchy
`f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi`

![Example](samples/Another_gem_from_Cauchy.png)

### An unneccesary number of scripts
`x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}`

![Example](samples/An_unneccesary_number_of_scripts.png)

### Quartic Function
`\mathop{\overbrace{c_4x^4 + c_3x^3 + c_2x^2 + c_1x + c_0}}\limits^{\gray{\mathrm{Quartic}}}`

![Example](samples/Quartic_Function.png)

### Another fun identity
`3^3 + 4^4 + 3^3 + 5^5 = 3435`

![Example](samples/Another_fun_identity.png)

# Usage

## Simple example

You can see a simple example of use in [examples/ttfparser_cairo.rs](examples/ttfparser_cairo.rs). To run this example, run the following in the root of the repository.

```bash
cargo r --example ttfparser-cairo --features cairo-renderer,ttfparser-fontparser
```
The program will output `test.svg`.

## More generally

In a nutshell, rendering a formula requires:

  - Parsing the formula into `ParseNode` (using `rex::parser::engine::parse`).
  - Creating a `FontContext` struct from a certain font struct provided by the crate.
  - Creating a `LayoutSettings` struct from this font context, specifying font size and font context.
  - Creating a `Layout`.
  - Creating a `LayoutNode` from `ParseNode` using `rex::layout::engine::layout`.
  - Add the node to the `Layout` using `Layout::add_node`.
  - Create a `Renderer`;
  - Create the relevant renderer backend (e.g. `CairoBackend::new` for cairo).
  - Call `Renderer::render` with the layout and the backend as arguments.

## Intended use: as a library

Add the following to `Cargo.toml`:

```toml
rex = {git = "https://github.com/KenyC/ReX", features = [<whatever features you deem relevant>]}
```

# License

## Fork

Any modifications made in this fork fork is distributed under the MIT license. See LICENSE for details.

## Original

The original ReX is primarily distributed under the terms of both the MIT license and
the Apache License (Version 2.0), with portions covered by various BSD-like
licenses. 

*Note (Keny C):* The license files were not provided in the original repository. The problem was raised [here](https://github.com/ReTeX/ReX/issues/39)). Given lack of reply, I'm not sure which parts are licensed by what.
