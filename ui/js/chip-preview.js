/**
 * KeyViewer Chip Preview
 * Unified chip preview rendering
 */

class ChipPreview {
    constructor(options) {
        this.containerId = options.containerId;
        this.getStyleGroups = options.getStyleGroups || (() => []);
        this.getOverlaySettings = options.getOverlaySettings || (() => ({}));
        
        this.container = document.getElementById(this.containerId);
    }
    
    /**
     * Render preview chips
     * @param {Object} options - Render options
     */
    render(options = {}) {
        if (!this.container) return;
        
        const {
            labels = ['Q', 'W', 'E', 'R', 'A', 'S', 'D', 'F'],
            maxChips = 8,
            showBackground = true
        } = options;
        
        const styleGroups = this.getStyleGroups();
        const overlaySettings = this.getOverlaySettings();
        
        // Find "all" type group for default chip style
        const allGroup = styleGroups.find(g => g.type === 'all');
        
        // Get default chip styles
        const chipBg = allGroup ? getGroupBackgroundStyle(allGroup) : (overlaySettings.chip_bg || '#000000');
        const isImage = allGroup && allGroup.bgMode === 'image' && allGroup.image;
        
        const styles = {
            gap: overlaySettings.chip_gap || 8,
            paddingV: overlaySettings.chip_pad_v || 10,
            paddingH: overlaySettings.chip_pad_h || 14,
            radius: overlaySettings.chip_radius || 10,
            fontSize: overlaySettings.chip_font_px || 24,
            fontWeight: overlaySettings.chip_font_weight || 700,
            color: overlaySettings.chip_fg || '#ffffff',
            background: overlaySettings.background || 'rgba(0,0,0,0)',
            align: overlaySettings.align || 'left',
            direction: overlaySettings.direction || 'ltr'
        };
        
        // Apply container styles
        if (showBackground) {
            this.container.style.background = styles.background;
        }
        
        // Find queue container
        const queue = this.container.querySelector('.queue') || this.container;
        queue.style.display = 'flex';
        queue.style.flexWrap = 'wrap';
        queue.style.gap = `${styles.gap}px`;
        queue.style.justifyContent = styles.align === 'center' ? 'center' : 
                                     styles.align === 'right' ? 'flex-end' : 'flex-start';
        queue.style.direction = styles.direction;
        
        // Clear and render chips
        queue.innerHTML = '';
        
        const count = Math.min(labels.length, maxChips);
        for (let i = 0; i < count; i++) {
            const chip = document.createElement('div');
            chip.className = 'chip';
            chip.textContent = labels[i];
            
            applyChipStyles(chip, {
                background: chipBg,
                isImage: isImage,
                color: styles.color,
                paddingV: styles.paddingV,
                paddingH: styles.paddingH,
                borderRadius: styles.radius,
                fontSize: styles.fontSize,
                fontWeight: styles.fontWeight,
                minWidth: 44,
                textAlign: 'center',
                fontFamily: 'ui-sans-serif, system-ui, "Segoe UI", Roboto, Arial'
            });
            
            queue.appendChild(chip);
        }
    }
    
    /**
     * Render specific key chips for unified style preview
     * @param {Object} options
     */
    renderStylePreview(options = {}) {
        if (!this.container) return;
        
        const {
            chips = [
                { label: 'A', type: 'normal' },
                { label: 'TAB', type: 'medium' },
                { label: 'LSHIFT', type: 'wide' }
            ],
            styleGroups = [],
            chipSettings = {}
        } = options;
        
        // Find "all" type group
        const allGroup = styleGroups.find(g => g.type === 'all');
        const chipBg = allGroup ? getGroupBackgroundStyle(allGroup) : (chipSettings.bg || '#000000');
        const isImage = allGroup && allGroup.bgMode === 'image' && allGroup.image;
        
        this.container.style.display = 'flex';
        this.container.style.flexWrap = 'wrap';
        this.container.style.gap = `${chipSettings.gap || 8}px`;
        this.container.style.justifyContent = 'center';
        this.container.innerHTML = '';
        
        chips.forEach(chipDef => {
            const chip = document.createElement('div');
            chip.className = 'chip';
            chip.textContent = chipDef.label;
            
            // Calculate width multiplier for different key types
            let widthMultiplier = 1;
            if (chipDef.type === 'medium') widthMultiplier = 1.5;
            if (chipDef.type === 'wide') widthMultiplier = 2;
            
            const baseWidth = 44;
            const paddingH = chipSettings.padH || 14;
            
            applyChipStyles(chip, {
                background: chipBg,
                isImage: isImage,
                color: chipSettings.fg || '#ffffff',
                paddingV: chipSettings.padV || 10,
                paddingH: paddingH,
                borderRadius: chipSettings.radius || 10,
                fontSize: chipSettings.fontSize || 24,
                fontWeight: chipSettings.fontWeight || 700,
                minWidth: baseWidth * widthMultiplier,
                textAlign: 'center',
                fontFamily: 'ui-sans-serif, system-ui, "Segoe UI", Roboto, Arial'
            });
            
            this.container.appendChild(chip);
        });
    }
}

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ChipPreview };
}

