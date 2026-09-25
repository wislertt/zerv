Output zerv-preview.gif
Set Shell "zsh"
Set FontSize 18
Set Width 960
Set Height 640
Set Theme {"background":"#0C161D","foreground":"#E8DFC8","cursor":"#FBBF24","cursor_text":"#0C161D","selection_background":"#B45309","selection_foreground":"#FDEBC8","palette":["#1F2E38","#E06C5E","#6FBF73","#FBBF24","#5CA8DB","#C88AD2","#62C1D8","#E8DFC8","#3E525F","#EC8B7D","#8FD48A","#FCD34D","#82BCE6","#D8A8E0","#7FD0E4","#F5EDD8"]}
Set Padding 42
Set CursorBlink false

# beat 1: tagged release on main
Type "echo 'on main, tagged v1.2.3'"
Enter
Sleep 1000ms
Type "git log --oneline -1 && git tag"
Enter
Sleep 1000ms
Type "zerv flow"
Enter
Sleep 3s

# beat 2: work lands on develop
Hide
Type "cd ../wt-develop && clear"
Enter
Sleep 1s
Show
Type "echo 'new branch: develop'"
Enter
Sleep 1000ms
Type "git log --oneline -1"
Enter
Sleep 1000ms
Type "zerv flow"
Enter
Sleep 3s

# beat 3: feature branch off develop
Hide
Type "cd ../wt-feature && clear"
Enter
Sleep 1s
Show
Type "echo 'new branch: feature/otp'"
Enter
Sleep 1000ms
Type "git log --oneline -1"
Enter
Sleep 1000ms
Type "zerv flow"
Enter
Sleep 3s

Enter

# beat 4: uncommitted changes
Type "echo 'uncommitted changes'"
Enter
Sleep 1000ms
Type "echo '// wip' >> main.rs"
Enter
Sleep 1000ms
Type "git status -s"
Enter
Sleep 1000ms
Type "zerv flow"
Enter
Sleep 3s

Sleep 3s
