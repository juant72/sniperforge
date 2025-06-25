# PowerShell script to fix Unicode issues in CLI output
$filePath = "src\cli.rs"

# Read the file content
$content = Get-Content -Path $filePath -Raw

# Replace emoji and special Unicode characters with ASCII equivalents
$replacements = @{
    "🚀" = "[START]"
    "📊" = "[STATS]"
    "🧪" = "[TEST]"
    "⚙️" = "[CONFIG]"
    "💰" = "[WALLET]"
    "🎯" = "[TARGET]"
    "📈" = "[UP]"
    "🔍" = "[SEARCH]"
    "⚡" = "[FAST]"
    "🤖" = "[ML]"
    "🔄" = "[REFRESH]"
    "✅" = "[OK]"
    "❌" = "[FAIL]"
    "⚠️" = "[WARN]"
    "🟢" = "[GREEN]"
    "🔗" = "[LINK]"
    "💾" = "[SAVE]"
    "🎮" = "[GAME]"
    "🏃" = "[RUN]"
    "🎉" = "[SUCCESS]"
    "🪐" = "[JUPITER]"
    "👛" = "[WALLET]"
    "🏗️" = "[BUILD]"
    "🌐" = "[NET]"
    "💸" = "[COST]"
    "📝" = "[NOTE]"
    "💧" = "[DROP]"
    "🔑" = "[KEY]"
    "🔌" = "[CONNECT]"
    "⏳" = "[WAIT]"
    "🧠" = "[BRAIN]"
    "📋" = "[COPY]"
    "⚖️" = "[SCALE]"
    "🛡️" = "[SHIELD]"
    "🔮" = "[PREDICT]"
    "📉" = "[DOWN]"
    "💡" = "[IDEA]"
    "🏆" = "[TROPHY]"
    "🔧" = "[TOOL]"

    # Box drawing characters
    "┌" = "+"
    "┐" = "+"
    "└" = "+"
    "┘" = "+"
    "├" = "+"
    "┤" = "+"
    "┬" = "+"
    "┴" = "+"
    "┼" = "+"
    "│" = "|"
    "─" = "-"
    "━" = "="
    "═" = "="

    # Bullet point
    "•" = "*"
    "‣" = "-"
    "▲" = "^"
    "▼" = "v"

    # Other special chars
    "…" = "..."
}

# Apply all replacements
foreach ($old in $replacements.Keys) {
    $new = $replacements[$old]
    $content = $content.Replace($old, $new)
}

# Write the fixed content back
Set-Content -Path $filePath -Value $content -Encoding UTF8

Write-Host "Unicode characters have been replaced with ASCII equivalents in $filePath"
