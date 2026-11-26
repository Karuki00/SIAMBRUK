# SIAMBRUK — Anti-AI-Slop Design and Engineering Skill

## Purpose

This skill defines the design and implementation standards for SIAMBRUK.

The objective is to produce a professional, practical, coherent inventory management application rather than a generic AI-generated dashboard.

Every design and implementation decision must serve a real business purpose.

## 1. Design Philosophy

Follow these principles:

- Functional before decorative.
- Clear before clever.
- Consistent before experimental.
- Practical before visually impressive.
- Data accuracy before visual effects.
- Restraint before excessive decoration.

SIAMBRUK is a business management tool. It should feel like software someone could realistically use to manage daily operations.

## 2. Avoid Generic AI-Generated Design

Do not automatically use:

- Purple-to-blue gradients.
- Excessive glassmorphism.
- Large rounded cards everywhere.
- Excessive shadows.
- Decorative background blobs.
- Oversized headings.
- Random colorful icons.
- Unnecessary animated elements.
- Fake analytics charts.
- Large empty dashboard sections.
- Excessive badges and pills.
- Decorative statistics without business meaning.
- Identical layouts for every page.

Do not use visual effects merely because they look modern.

## 3. Visual Identity

Create a recognizable and consistent visual identity for SIAMBRUK.

Use:

- A restrained color palette.
- A primary accent color with a clear purpose.
- Neutral surfaces for most content.
- Consistent typography.
- Clear spacing and alignment.
- Consistent borders and corner radii.
- Semantic colors for success, warning, and error states.

Do not introduce new colors without a reason.

Use icons consistently and only when they improve recognition or usability.

## 4. Layout and Navigation

Use a practical desktop-oriented layout.

The application should have clear navigation for:

- Dashboard.
- Products.
- Categories.
- Inventory.
- Purchases.
- Sales.
- Reports.
- SMART Ranking.
- Settings.

Navigation must be predictable.

Avoid unnecessary nested navigation and overly complicated page structures.

Keep page headings, actions, filters, and content organized consistently.

## 5. Tables and Data Presentation

Inventory software is data-heavy. Tables are often more useful than decorative cards.

Use tables for:

- Products and variants.
- Inventory quantities.
- Purchase records.
- Sales history.
- Stock movements.
- SMART rankings.
- Reports.

Tables must have:

- Clear column labels.
- Appropriate alignment.
- Readable numerical values.
- Consistent formatting.
- Useful sorting and filtering where appropriate.
- Clear empty states.
- Understandable row actions.

Avoid displaying important operational information in tiny text.

Do not hide useful information behind unnecessary interactions.

## 6. Forms and Interactions

Forms must be designed for actual business operations.

Requirements:

- Clear field labels.
- Appropriate input types.
- Validation before submission.
- Helpful error messages.
- Sensible defaults.
- Clear required-field indicators.
- Confirmation for destructive actions.
- Feedback after successful operations.

Do not use decorative or nonfunctional buttons.

Every visible action must work or be clearly marked as unavailable.

Avoid unnecessary multi-step forms for simple operations.

## 7. Dashboard Quality

The dashboard must answer useful business questions.

Examples:

- How many products and variants exist?
- How much stock is available?
- How much was sold today?
- What is the revenue?
- What is the estimated profit?
- Which variants are running low?
- Which variants are performing well?

Use charts only when they reveal meaningful patterns.

Do not add charts just to make the dashboard look sophisticated.

Every metric must be calculated from real application data or explicitly labeled demonstration data.

## 8. SMART Ranking Design Standards

The SMART module is an important academic and functional component of SIAMBRUK.

It must be visually understandable and mathematically transparent.

### Fixed criteria

| Criterion | Weight | Classification |
|---|---:|---|
| Keuntungan | 30% | Benefit |
| Produk Terjual | 40% | Benefit |
| Stok | 15% | Cost |
| Harga Pembelian | 15% | Cost |

Each product variant must receive an independent evaluation and ranking.

Do not combine variants of the same product into one SMART alternative.

### Explain the calculation

The interface must distinguish between:

1. Raw criterion values.
2. Criterion weights.
3. Normalized utility values.
4. Weighted contributions.
5. Final SMART scores.
6. Ranking positions.

Users must be able to understand why a variant receives its score.

Do not display arbitrary scores or rankings that are not calculated from actual records.

### Benefit and cost direction

Benefit criteria:

- Higher profit produces a higher utility.
- More units sold produces a higher utility.

Cost criteria:

- Lower remaining stock produces a higher utility.
- Lower purchase price produces a higher utility.

Use the correct normalization formulas and handle equal values safely.

### SMART visual presentation

The ranking page should prioritize:

- Variant identity.
- Final score.
- Ranking position.
- Important raw values.
- Evaluation period.
- A way to inspect the detailed calculation.

A detailed evaluation view should show the contribution of each criterion to the final score.

Do not rely only on colorful progress bars or unexplained percentage badges.

Do not imply that a high SMART score automatically means a product should be purchased or restocked. The system provides decision support, not automatic decisions.

## 9. Empty, Loading, and Error States

Every major page must handle:

- No records.
- Loading data.
- Failed operations.
- Invalid input.
- Missing information.
- Database errors.

Empty states should explain what the user can do next.

Do not fill empty states with fabricated business information.

Demonstration data must be clearly identified.

## 10. Consistency

Maintain consistent:

- Typography.
- Spacing.
- Colors.
- Button styles.
- Form controls.
- Table styles.
- Modal behavior.
- Navigation.
- Error handling.
- Number and currency formatting.

Do not redesign individual pages independently without considering the whole application.

## 11. Engineering Quality

Anti-AI-slop applies to code as well as appearance.

Avoid:

- Overengineering.
- Unnecessary abstractions.
- Repeated code.
- Giant components.
- Business logic embedded in presentation components.
- Fake functionality.
- Hardcoded dashboard numbers.
- Silent calculation assumptions.
- Unhandled errors.
- Unnecessary dependencies.
- Unexplained magic numbers.

Use clear names, sensible module boundaries, TypeScript types, validation, and testable business logic.

Keep SMART calculations in a dedicated, independently testable module.

Use database transactions for operations that modify multiple related records.

## 12. Implementation Discipline

Before implementing a feature:

1. Understand its business purpose.
2. Check the existing architecture.
3. Identify the data it requires.
4. Define the expected behavior.
5. Consider validation and edge cases.
6. Implement it consistently with existing patterns.
7. Test it.
8. Verify that it does not break existing features.

Do not rewrite working parts of the project unnecessarily.

Do not claim a feature is complete until its actual functionality has been tested.

## 13. Final Standard

The final SIAMBRUK application should feel like a coherent, purpose-built inventory management system.

It should not look like a generic dashboard template with a business name attached.

It should be understandable to a small-business operator, technically maintainable by a developer, and academically explainable as a real information system with a properly implemented SMART decision-support method.

# Additional Design Skill — SIAMBRUK 2D Interactive Experience

## 14. 3D Simulation Environment Design Standards

SIAMBRUK includes a 3D interactive shop or warehouse.

This environment must look like a deliberately designed management simulation game, not a technical prototype.

### Visual quality

Prioritize:
- A complete room with proper walls and flooring.
- Recognizable shelves and storage furniture.
- Properly modeled product packages.
- Consistent object proportions.
- Good lighting and shadows.
- Depth, perspective, and spatial composition.
- A coherent low-poly or stylized 3D aesthetic.

Do not settle for primitive cubes placed on flat shelves.

Do not leave large empty areas without purpose.

Do not use a plain white background as a substitute for an environment.

### Interaction quality

Objects must communicate that they can be interacted with.

Use subtle hover highlighting, selection indicators, and useful information panels.

Camera movement must feel responsive and predictable.

The user should never feel trapped inside the environment.

### Game-like experience

Make the environment feel like a small shop simulation through:
- First-person exploration.
- Interactive shelves.
- Product selection.
- Restocking animations.
- Sales animations.
- Subtle ambient details.

However, do not add unnecessary game mechanics, points, levels, quests, or fictional currencies unless explicitly requested.

This is a real business application with a game-like interface.

### Business integrity

The 3D scene must always reflect real database values.

A package must never appear or disappear simply because an animation played.

Inventory operations must complete successfully before visual changes occur.

All interactions must use existing inventory and sales services.

### Performance and usability

Keep the scene lightweight and responsive.

Do not prioritize visual complexity over usability.

Provide standard inventory interfaces as an alternative to the 3D experience.

The result must be suitable for everyday use as well as a university project demonstration.