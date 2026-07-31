Output assets/demo.gif

Set Shell bash
Set FontSize 15
Set Width 1200
Set Height 480
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
Sleep 1800ms

# walk the stash list; the diff pane follows the selection
Down
Sleep 900ms
Down
Sleep 1200ms

# drill into "retry + abort for api client" - a multi-file stash - and see its
# file tree: paths grouped by directory, each tagged with +added -deleted
Right
Sleep 1800ms

# open the first file's diff
Right
Sleep 1200ms

# scroll it with plain arrow keys
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Sleep 700ms
Up
Sleep 250ms
Up
Sleep 800ms

# pop just this one file, leaving the other stashed (declined here, so the
# seeded demo repo stays reusable without needing to be reseeded)
Type "p"
Sleep 1300ms
Type "n"
Sleep 900ms

# back out: file diff -> tree -> stash list
Left
Sleep 1200ms
Left
Sleep 1200ms

# popping the whole stash still works the same way
Enter
Sleep 1500ms
Type "n"
Sleep 900ms

Type "q"
Sleep 600ms
