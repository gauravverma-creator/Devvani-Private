; ModuleID = 'parichaya'
source_filename = "parichaya"

@fmtscan = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@fmt = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@fmt.1 = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1

define i64 @_devvani_main() {
entry:
  %scanftmp = alloca i64, align 8
  %scanftmpcall = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmtscan, i32 0, i32 0), i64* %scanftmp)
  %scanload = load i64, i64* %scanftmp, align 4
  %printftmp = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @fmt, i32 0, i32 0), i64 %scanload)
  %printftmp1 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @fmt.1, i32 0, i32 0), i64 8)
  ret i64 0
}

declare i32 @scanf(i8*, ...)

declare i32 @printf(i8*, ...)
