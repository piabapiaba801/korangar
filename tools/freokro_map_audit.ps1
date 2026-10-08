param(
    [Parameter(Mandatory = $true)][string] $DataDirectory,
    [string] $ReportDirectory = (Join-Path $PSScriptRoot '..\reports')
)

$ErrorActionPreference = 'Stop'
$mapDirectory = (Resolve-Path -LiteralPath $DataDirectory).Path
New-Item -ItemType Directory -Path $ReportDirectory -Force | Out-Null
$encoding = [Text.Encoding]::GetEncoding(949)
$textureUsers = @{}

function Read-MapHeader([string] $path, [string] $signature) {
    if (-not [IO.File]::Exists($path)) { return $null }
    $stream = [IO.File]::OpenRead($path)
    try {
        $reader = [IO.BinaryReader]::new($stream)
        if ($encoding.GetString($reader.ReadBytes(4)) -ne $signature) { return $null }
        $major = $reader.ReadByte()
        $minor = $reader.ReadByte()
        $width = $reader.ReadInt32()
        $height = $reader.ReadInt32()
        $result = @{ Version = "$major.$minor"; Width = $width; Height = $height; Textures = @() }
        if ($signature -eq 'GRGN') {
            $reader.ReadSingle() | Out-Null
            $count = $reader.ReadInt32()
            $length = $reader.ReadInt32()
            if ($count -lt 0 -or $count -gt 100000 -or $length -lt 1 -or $length -gt 1024) {
                throw "Invalid texture table in $path"
            }
            for ($index = 0; $index -lt $count; $index++) {
                $bytes = $reader.ReadBytes($length)
                $result.Textures += ($encoding.GetString($bytes).Split([char]0)[0])
            }
        }
        return $result
    } finally {
        $stream.Dispose()
    }
}

function Read-RswReferences([string] $path) {
    $stream = [IO.File]::OpenRead($path)
    try {
        $reader = [IO.BinaryReader]::new($stream)
        if ($encoding.GetString($reader.ReadBytes(4)) -ne 'GRSW') { return $null }
        $major = $reader.ReadByte()
        $minor = $reader.ReadByte()
        if ($major -gt 2 -or ($major -eq 2 -and $minor -ge 5)) { $reader.ReadInt32() | Out-Null }
        if ($major -gt 2 -or ($major -eq 2 -and $minor -ge 2)) { $reader.ReadByte() | Out-Null }
        $reader.ReadBytes(40) | Out-Null # INI file
        $ground = $encoding.GetString($reader.ReadBytes(40)).Split([char]0)[0]
        $gat = $encoding.GetString($reader.ReadBytes(40)).Split([char]0)[0]
        return @{ Ground = $ground; Gat = $gat; Version = "$major.$minor" }
    } finally {
        $stream.Dispose()
    }
}

$inventory = foreach ($rsw in Get-ChildItem -LiteralPath $mapDirectory -Filter '*.rsw' -File) {
    $id = $rsw.BaseName
    $references = Read-RswReferences $rsw.FullName
    $gndName = if ($references -and $references.Ground) { $references.Ground } else { "$id.gnd" }
    $gatName = if ($references -and $references.Gat) { $references.Gat } else { "$id.gat" }
    $gndPath = Join-Path $mapDirectory $gndName
    $gatPath = Join-Path $mapDirectory $gatName
    $gnd = Read-MapHeader $gndPath 'GRGN'
    $gat = Read-MapHeader $gatPath 'GRAT'
    if ($gnd) {
        foreach ($name in $gnd.Textures) {
            if (-not $textureUsers.ContainsKey($name)) { $textureUsers[$name] = [Collections.Generic.List[string]]::new() }
            $textureUsers[$name].Add($id)
        }
    }
    $bytes = $rsw.Length
    if ([IO.File]::Exists($gndPath)) { $bytes += ([IO.FileInfo]::new($gndPath)).Length }
    if ([IO.File]::Exists($gatPath)) { $bytes += ([IO.FileInfo]::new($gatPath)).Length }
    [pscustomobject]@{
        map_id = $id
        rsw = "data/$id.rsw"
        gnd = if ($gnd) { "data/$gndName" } else { '' }
        gat = if ($gat) { "data/$gatName" } else { '' }
        gnd_width = if ($gnd) { $gnd.Width } else { '' }
        gnd_height = if ($gnd) { $gnd.Height } else { '' }
        gat_width = if ($gat) { $gat.Width } else { '' }
        gat_height = if ($gat) { $gat.Height } else { '' }
        ground_texture_count = if ($gnd) { $gnd.Textures.Count } else { '' }
        object_references = 'not_yet_parsed'
        load_status = 'not_yet_tested'
        compatibility = 'not_yet_tested'
        source_bytes = $bytes
        classification = if ($id -eq 'prontera') { 'pilot_city' } else { 'unclassified' }
    }
}
$inventory | Export-Csv -LiteralPath (Join-Path $ReportDirectory 'map_inventory.csv') -NoTypeInformation -Encoding utf8

$dependencies = foreach ($name in ($textureUsers.Keys | Sort-Object)) {
    $path = Join-Path (Join-Path $mapDirectory 'texture') $name
    $file = if ([IO.File]::Exists($path)) { [IO.FileInfo]::new($path) } else { $null }
    [pscustomobject]@{
        path = "data/texture/$($name.Replace('\', '/'))"
        type = 'ground_texture'
        bytes = if ($file) { $file.Length } else { '' }
        sha256 = 'not_yet_hashed'
        consumers = ($textureUsers[$name] | Sort-Object -Unique) -join ';'
        scope = 'unknown'
        ownership = 'not_verified'
    }
}
$dependencies | Export-Csv -LiteralPath (Join-Path $ReportDirectory 'asset_dependencies.csv') -NoTypeInformation -Encoding utf8

Write-Output "Maps indexed: $(@($inventory).Count)"
Write-Output "Distinct ground texture references: $(@($dependencies).Count)"
