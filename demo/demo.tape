Output assets/demo.gif

Set Shell bash
Set FontSize 15
Set Width 1200
Set Height 470
Set Padding 12
Set Theme "Catppuccin Mocha"
Set Framerate 24
Set PlaybackSpeed 1.0

Hide
Type "cd /tmp/lazystash-demo && clear"
Enter
Sleep 500ms
Show

Type "lazystash"
Sleep 400ms
Enter
Sleep 2s

# walk the stash list; the diff pane follows the selection
Down
Sleep 1200ms
Down
Sleep 1200ms
Down
Sleep 1500ms

# scroll the diff
Ctrl+d
Sleep 1200ms
Ctrl+u
Sleep 900ms

# drill into that stash's files, step through them
Right
Sleep 1500ms
Down
Sleep 1500ms
Left
Sleep 1200ms

# offer to pop, then back out - nothing is mutated in the demo
Enter
Sleep 1800ms
Type "n"
Sleep 1200ms

Type "q"
Sleep 800ms
