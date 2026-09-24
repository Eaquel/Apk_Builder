-keepclasseswithmembernames class com.apk.builder.Builder {
    native <methods>;
}
-keep class com.apk.builder.Builder { *; }

-keep class com.apk.builder.BuilderActivity { *; }
-keep class com.apk.builder.BuilderApplication { *; }

-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile
