/**
 * KeyViewer Gradient Editor
 * Reusable gradient editor component
 */

class GradientEditor {
    constructor(options) {
        this.containerId = options.containerId;
        this.previewId = options.previewId;
        this.angleId = options.angleId;
        this.angleNumId = options.angleNumId;
        this.stopsId = options.stopsId;
        this.addBtnId = options.addBtnId;
        this.onChange = options.onChange || (() => {});
        
        this.stops = createDefaultGradientStops();
        this.angle = 180;
        
        this.init();
    }
    
    init() {
        this.preview = document.getElementById(this.previewId);
        this.angleSlider = document.getElementById(this.angleId);
        this.angleNum = document.getElementById(this.angleNumId);
        this.stopsContainer = document.getElementById(this.stopsId);
        this.addBtn = document.getElementById(this.addBtnId);
        
        this.bindEvents();
        this.render();
    }
    
    bindEvents() {
        if (this.angleSlider && this.angleNum) {
            this.angleSlider.addEventListener('input', () => {
                this.angleNum.value = this.angleSlider.value;
                this.angle = parseInt(this.angleSlider.value);
                this.updatePreview();
                this.onChange();
            });
            
            this.angleNum.addEventListener('input', () => {
                this.angleSlider.value = this.angleNum.value;
                this.angle = parseInt(this.angleNum.value);
                this.updatePreview();
                this.onChange();
            });
        }
        
        if (this.addBtn) {
            this.addBtn.addEventListener('click', () => this.addStop());
        }
    }
    
    render() {
        if (!this.stopsContainer) return;
        
        this.stopsContainer.innerHTML = this.stops.map((stop, idx) => `
            <div class="grad-stop-row" style="display:flex; gap:4px; align-items:center" data-idx="${idx}">
                <input type="color" value="${stop.color}" style="width:32px;height:24px" data-action="color" data-idx="${idx}"/>
                <input type="text" value="${stop.color}" style="flex:1;font-family:monospace;font-size:11px" data-action="colorText" data-idx="${idx}"/>
                <input type="number" min="0" max="100" value="${stop.position}" style="width:45px;font-size:11px" data-action="position" data-idx="${idx}"/>
                <span style="font-size:10px">%</span>
                ${this.stops.length > 2 ? `<button data-action="remove" data-idx="${idx}" style="padding:1px 4px;font-size:11px;color:#ff5b5b">✕</button>` : ''}
            </div>
        `).join('');
        
        // Bind stop events
        this.stopsContainer.querySelectorAll('[data-action]').forEach(el => {
            const action = el.dataset.action;
            const idx = parseInt(el.dataset.idx);
            
            if (action === 'color' || action === 'colorText') {
                el.addEventListener('input', (e) => {
                    const value = e.target.value;
                    if (action === 'colorText' && !isValidHex(value)) return;
                    this.updateStop(idx, 'color', value);
                    // Sync color picker and text
                    const row = el.closest('.grad-stop-row');
                    if (row) {
                        const colorInput = row.querySelector('[data-action="color"]');
                        const textInput = row.querySelector('[data-action="colorText"]');
                        if (colorInput && action === 'colorText') colorInput.value = value;
                        if (textInput && action === 'color') textInput.value = value;
                    }
                });
            } else if (action === 'position') {
                el.addEventListener('input', (e) => {
                    this.updateStop(idx, 'position', parseInt(e.target.value) || 0);
                });
            } else if (action === 'remove') {
                el.addEventListener('click', () => this.removeStop(idx));
            }
        });
        
        this.updatePreview();
    }
    
    updateStop(idx, key, value) {
        if (key === 'position') {
            value = Math.max(0, Math.min(100, value));
        }
        this.stops[idx][key] = value;
        this.updatePreview();
        this.onChange();
    }
    
    addStop() {
        const lastPos = this.stops[this.stops.length - 1]?.position || 100;
        const newPos = Math.min(100, Math.round(lastPos / 2 + 25));
        this.stops.push({ color: '#666666', position: newPos });
        this.stops.sort((a, b) => a.position - b.position);
        this.render();
        this.onChange();
    }
    
    removeStop(idx) {
        if (this.stops.length <= 2) return;
        this.stops.splice(idx, 1);
        this.render();
        this.onChange();
    }
    
    updatePreview() {
        if (!this.preview) return;
        this.preview.style.background = this.getCSS();
    }
    
    getCSS() {
        return generateGradientCSS(this.stops, this.angle);
    }
    
    getStops() {
        return [...this.stops];
    }
    
    getAngle() {
        return this.angle;
    }
    
    setData(stops, angle) {
        if (stops && stops.length >= 2) {
            this.stops = [...stops];
        } else {
            this.stops = createDefaultGradientStops();
        }
        this.angle = angle || 180;
        
        if (this.angleSlider) this.angleSlider.value = this.angle;
        if (this.angleNum) this.angleNum.value = this.angle;
        
        this.render();
    }
    
    reset() {
        this.stops = createDefaultGradientStops();
        this.angle = 180;
        if (this.angleSlider) this.angleSlider.value = 180;
        if (this.angleNum) this.angleNum.value = 180;
        this.render();
    }
}

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { GradientEditor };
}

