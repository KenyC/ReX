import os
import subprocess

formulas = [
    (r'x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}', 'The_Quadratic_Formula'),
    (r'\sin(\theta + \phi) = \sin(\theta)\cos(\phi) + \sin(\phi)\cos(\theta)', 'Double_angle_formula_for_Sine'),
    (r'\int_D (\nabla \cdot F)\,\mathrm{d}V = \int_{\partial D} F \cdot n\,\mathrm{d}S', 'Divergence_Theorem'),
    (r'\sigma = \sqrt{ \frac{1}{N} \sum_{i=1}^N (x_i - \mu)^2 }', 'Standard_Deviation'),
    (r'f(x) = \int_{-\infty}^{\infty} \hat f(\xi) e^{2\pi i \xi x}\,\mathrm{d}\xi', 'Fourier_Inverse'),
    (r'\left\vert \sum_k a_kb_k \right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}', 'Cauchy-Schwarz_Inequality'),
    (r'e = \lim_{n \to \infty} \left(1 + \frac{1}{n}\right)^n', 'Exponent'),
    (r'\frac{1}{\pi} = \frac{2\sqrt{2}}{9801} \sum_{k=0}^\infty \frac{ (4k)! (1103+26390k) }{ (k!)^4 396^{4k} }', 'Ramanujan\'s_Identity'),
    (r'\int_{-\infty}^{\infty} \frac{\sin(x)}{x}\,\mathrm{d}x = \int_{-\infty}^{\infty}\frac{\sin^2(x)}{x^2}\,\mathrm{d}x', 'A_surprising_identity'),
    (r'\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \cdots}}}}', 'Another_gem_from_Ramanujan'),
    (r'f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi', 'Another_gem_from_Cauchy'),
    (r'x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}', 'An_unneccesary_number_of_scripts'),
    (r'\mathop{\overbrace{c_4x^4 + c_3x^3 + c_2x^2 + c_1x + c_0}}\limits^{\gray{\mathrm{Quartic}}}', 'Quartic_Function'),
    (r'3^3 + 4^4 + 3^3 + 5^5 = 3435', 'Another_fun_identity'),
]

output_dir = 'samples'

# Create the output directory if it doesn't exist
if not os.path.exists(output_dir):
    os.makedirs(output_dir)

for formula, name in formulas:
    # Generate the SVG file
    svg_file = os.path.join(output_dir, f'{name}.svg')
    command = f'cargo r --example svg-basic --features="cairo-renderer ttfparser-fontparser" -- "{formula}" -o "{svg_file}"'
    subprocess.run(command, shell=True, check=True)

    # Convert SVG to PNG using an external tool (e.g., Inkscape)
    png_file = os.path.join(output_dir, f'{name}.png')
    command = f'inkscape --export-png="{png_file}" "{svg_file}"'
    subprocess.run(command, shell=True, check=True)

print('Render generation complete.')
