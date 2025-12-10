# Downloads a small PBR texture set (albedo, normal, roughness, AO) from Poly Haven
# into assets/pbr/<material>/ in this repo.
# Uses the Poly Haven API so URLs stay valid even if paths change.
# Edit the $materials list to choose which assets and resolution you want (1k/2k/4k/8k).

$ErrorActionPreference = "Stop"

# Pick assets from https://polyhaven.com/textures (IDs, not display names)
$materials = @(
    @{ name = "grass"; id = "aerial_grass_rock"; resolution = "1k" },
    @{ name = "sand";  id = "aerial_sand";       resolution = "1k" },
    @{ name = "dirt";  id = "aerial_ground_rock";resolution = "1k" },
    @{ name = "rock";  id = "rock_wall_12";      resolution = "1k" }
)

# Preferred map keys from the API (first match wins)
$mapPrefs = @{
    albedo    = @("Diffuse", "BaseColor");
    normal    = @("nor_gl", "Normal", "normal_gl");
    roughness = @("Rough", "roughness", "arm");   # if arm is used, roughness lives in G
    ao        = @("AO", "rough_ao", "Occlusion");
}

function Find-MapUrl {
    param(
        [object]$files,
        [string[]]$keys,
        [string]$resolution
    )

    foreach ($k in $keys) {
        if ($files.PSObject.Properties.Name -contains $k) {
            $resBlock = $files.$k.$resolution
            if ($null -ne $resBlock) {
                foreach ($format in @("png", "jpg", "exr")) {
                    if ($resBlock.PSObject.Properties.Name -contains $format) {
                        return $resBlock.$format.url
                    }
                }
            }
        }
    }
    return $null
}

foreach ($mat in $materials) {
    Write-Host "`n=== $($mat.name) ($($mat.id)) @ $($mat.resolution) ==="
    $destDir = Join-Path -Path "assets/pbr" -ChildPath $mat.name
    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    }

    $apiUrl = "https://api.polyhaven.com/files/$($mat.id)"
    $files = Invoke-RestMethod -Uri $apiUrl

    $targets = @(
        @{ key = "albedo";    dest = "albedo.png" },
        @{ key = "normal";    dest = "normal.png" },
        @{ key = "roughness"; dest = "roughness.png" },
        @{ key = "ao";        dest = "ao.png" }
    )

    foreach ($t in $targets) {
        $dest = Join-Path -Path $destDir -ChildPath $t.dest
        if (Test-Path $dest) {
            Write-Host "Skipping existing $dest"
            continue
        }

        $url = Find-MapUrl -files $files -keys $mapPrefs[$t.key] -resolution $mat.resolution
        if (-not $url) {
            Write-Warning "No map found for $($t.key) at $($mat.resolution) on $($mat.id)"
            continue
        }

        Write-Host "Downloading $url -> $dest"
        Invoke-WebRequest -Uri $url -OutFile $dest
    }
}

Write-Host "`nDone."
