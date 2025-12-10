/**
 * KeyViewer Utility Functions
 * Common utilities used across the application
 */

// ==================== Color Utilities ====================

/**
 * Check if a hex color string is valid
 * @param {string} hex - Color string to validate
 * @returns {boolean}
 */
function isValidHex(hex) {
    return /^#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})$/.test(hex);
}

/**
 * Check if a color is light (for text contrast)
 * @param {string} color - Hex color
 * @returns {boolean}
 */
function isLightColor(color) {
    if (!color || !color.startsWith('#')) return false;
    const hex = color.replace('#', '');
    const r = parseInt(hex.substr(0, 2), 16);
    const g = parseInt(hex.substr(2, 2), 16);
    const b = parseInt(hex.substr(4, 2), 16);
    return (r * 0.299 + g * 0.587 + b * 0.114) > 186;
}

// ==================== Gradient Utilities ====================

/**
 * Generate CSS gradient string from stops array
 * @param {Array} stops - Array of {color, position} objects
 * @param {number} angle - Gradient angle in degrees
 * @returns {string} CSS gradient string
 */
function generateGradientCSS(stops, angle = 180) {
    if (!stops || stops.length < 2) {
        return `linear-gradient(${angle}deg, #000000 0%, #333333 100%)`;
    }
    const sorted = [...stops].sort((a, b) => a.position - b.position);
    const stopsStr = sorted.map(s => `${s.color} ${s.position}%`).join(', ');
    return `linear-gradient(${angle}deg, ${stopsStr})`;
}

/**
 * Create default gradient stops
 * @returns {Array}
 */
function createDefaultGradientStops() {
    return [
        { color: '#000000', position: 0 },
        { color: '#333333', position: 100 }
    ];
}

// ==================== Chip Style Utilities ====================

/**
 * Get background style for a style group
 * @param {Object} group - Style group object
 * @returns {string} CSS background value
 */
function getGroupBackgroundStyle(group) {
    if (!group) return '#000000';
    
    if (group.bgMode === 'image' && group.image) {
        return `url(${group.image})`;
    } else if (group.bgMode === 'gradient') {
        if (group.gradStops && group.gradStops.length >= 2) {
            return generateGradientCSS(group.gradStops, group.gradAngle || 180);
        } else if (group.gradColor1 && group.gradColor2) {
            // Legacy support
            return `linear-gradient(180deg, ${group.gradColor1}, ${group.gradColor2})`;
        }
    } else if (group.bgMode === 'solid') {
        return group.bgColor || '#000000';
    }
    
    return group.bgColor || '#000000';
}

/**
 * Apply chip styles to an element
 * @param {HTMLElement} el - Target element
 * @param {Object} styles - Style configuration
 */
function applyChipStyles(el, styles) {
    if (!el || !styles) return;
    
    const {
        background,
        color,
        padding,
        paddingV,
        paddingH,
        borderRadius,
        fontSize,
        fontWeight,
        fontFamily,
        minWidth,
        textAlign,
        isImage
    } = styles;
    
    if (isImage && background) {
        el.style.backgroundImage = background.startsWith('url(') ? background : `url(${background})`;
        el.style.backgroundSize = 'cover';
        el.style.backgroundPosition = 'center';
        el.style.backgroundRepeat = 'no-repeat';
    } else if (background) {
        el.style.background = background;
    }
    
    if (color) el.style.color = color;
    if (padding) el.style.padding = padding;
    if (paddingV !== undefined && paddingH !== undefined) {
        el.style.padding = `${paddingV}px ${paddingH}px`;
    }
    if (borderRadius !== undefined) el.style.borderRadius = `${borderRadius}px`;
    if (fontSize !== undefined) el.style.fontSize = `${fontSize}px`;
    if (fontWeight !== undefined) el.style.fontWeight = fontWeight;
    if (fontFamily) el.style.fontFamily = fontFamily;
    if (minWidth !== undefined) el.style.minWidth = `${minWidth}px`;
    if (textAlign) el.style.textAlign = textAlign;
    
    // Common chip styles
    el.style.display = 'inline-flex';
    el.style.alignItems = 'center';
    el.style.justifyContent = 'center';
    el.style.boxShadow = 'inset 0 0 0 2px rgba(255,255,255,0.12), inset 0 -2px 4px rgba(0,0,0,0.25), inset 0 2px 4px rgba(255,255,255,0.08)';
}

// ==================== Sync Color Picker ====================

/**
 * Synchronize color picker and text input
 * @param {HTMLInputElement} picker - Color picker input
 * @param {HTMLInputElement} text - Text input
 * @param {Function} onChange - Optional callback on change
 */
function syncColorInputs(picker, text, onChange) {
    if (!picker || !text) return;
    
    picker.addEventListener('input', () => {
        text.value = picker.value;
        if (onChange) onChange(picker.value);
    });
    
    text.addEventListener('input', () => {
        if (isValidHex(text.value)) {
            picker.value = text.value;
            if (onChange) onChange(text.value);
        }
    });
}

// ==================== DOM Utilities ====================

/**
 * Safely get element value with fallback
 * @param {string} id - Element ID
 * @param {*} fallback - Fallback value
 * @returns {*}
 */
function getElementValue(id, fallback = '') {
    const el = document.getElementById(id);
    return el ? el.value : fallback;
}

/**
 * Safely set element value
 * @param {string} id - Element ID
 * @param {*} value - Value to set
 */
function setElementValue(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
}

// Export for module usage (if needed)
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        isValidHex,
        isLightColor,
        generateGradientCSS,
        createDefaultGradientStops,
        getGroupBackgroundStyle,
        applyChipStyles,
        syncColorInputs,
        getElementValue,
        setElementValue
    };
}

