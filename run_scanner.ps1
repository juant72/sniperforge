$input = "A"
$process = Start-Process -FilePath "cargo" -ArgumentList "run", "--bin", "arbitrage_bot" -PassThru -RedirectStandardInput $true -RedirectStandardOutput $true -RedirectStandardError $true
$process.StandardInput.WriteLine($input)
$process.StandardInput.Close()
$output = $process.StandardOutput.ReadToEnd()
$error = $process.StandardError.ReadToEnd()
$process.WaitForExit()
Write-Host $output
Write-Host $error
