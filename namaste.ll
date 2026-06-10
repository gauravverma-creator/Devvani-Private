; ModuleID = 'namaste'
source_filename = "namaste"

@fmt = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1

define i64 @_devvani_main() {
entry:
  %printftmp = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @fmt, i32 0, i32 0), i64 8)
  ret i64 0
}

declare i32 @printf(i8*, ...)
