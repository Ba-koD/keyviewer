# Convert ICO to PNG using .NET
Add-Type -AssemblyName System.Drawing

$icoPath = "src-tauri\icons\icon.ico"
$pngPath = "src-tauri\icons\icon.png"

try {
    # Load the ICO file
    $icon = [System.Drawing.Icon]::new($icoPath)
    
    # Convert to bitmap (use largest size available)
    $bitmap = $icon.ToBitmap()
    
    # Save as PNG
    $bitmap.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
    
    Write-Host "Successfully converted icon.ico to icon.png" -ForegroundColor Green
    
    # Cleanup
    $bitmap.Dispose()
    $icon.Dispose()
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}

