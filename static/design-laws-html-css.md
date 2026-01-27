# Design Laws: HTML/CSS Version

## Von Restorff Effect
- Use consistent spacing with padding and flexbox/grid layouts (flex-direction: column, gap properties)
- Apply clear typography hierarchy with font-size, font-weight for headings (e.g., h2 with bold styling)
- Add visual separators like <hr>, border properties, or translucent backgrounds to structure content

## Hick's Law
- Minimize visible options on each screen
- Use <details>/<summary>, checkboxes, or modal overlays for collapsing complexity
- Load complex filters or conditions only when needed (initially hidden with display: none)

## Jakob's Law
- Follow web conventions (e.g., <nav>, modal dialogs, tab navigation patterns)
- Keep "Add", edit, and delete buttons in expected places (e.g., top-right header or fixed toolbars)

## Fitts's Law
- Make important buttons large and clearly clickable with prominent button classes
- Ensure click targets are at least 44px × 44px
- Use icons (SVG or icon fonts) with readable sizing and proper labels

## Law of Proximity
- Group related controls using <fieldset>, <form>, or nested divs with gap/margin spacing
- Visually bundle related inputs using card-style containers with borders, shadows, or background colors

## Zeigarnik Effect
- Use progress bars, breadcrumb navigation, or "Step X of Y" text indicators in multi-step flows
- Show save states like "Saving..." text, notification banners, or toast alerts

## Goal-Gradient Effect
- Highlight the active step in workflows (e.g., with step indicators or emphasized navigation items)
- Use progress bars or styled primary buttons to encourage forward progress

## Law of Similarity
- Maintain consistent styling for checkboxes, buttons, filters, and badges using CSS classes
- Align icon sizes and spacing using reusable CSS classes or design tokens

## Miller's Law
- Break configuration into logical steps using <form> sections, tab interfaces, or collapsible <details> elements
- Default advanced options to collapsed state (details element closed by default)

## Doherty Threshold
- Keep UI interactions fast (<400ms) using efficient CSS, minimal DOM manipulation, and optimized rendering
- Display loading states with loading spinners, skeleton screens, or shimmer effects during data fetches
