#ifndef APK_BUILDER_NATIVE_BUILDER_HPP
#define APK_BUILDER_NATIVE_BUILDER_HPP

#include <jni.h>

#include <cstdint>

extern "C" {

uint32_t apk_builder_abi_version(void);

const char *apk_builder_core_version(void);

char *apk_builder_create_project(const char *root, const char *spec);

char *apk_builder_build_all(const char *root, const char *package_path,
                     const char *bundle_path, const char *key_path,
                     const char *key_password);

char *apk_builder_build_progress(void);

void apk_builder_build_expect(const char *timings);

void apk_builder_build_stop(void);

char *apk_builder_speak(const char *tag);

char *apk_builder_terminal_docs(const char *tag);

char *apk_builder_vibe(const char *request);

char *apk_builder_verify_self(const char *package_path,
                       const char *expected_certificate);

char *apk_builder_create_key(const char *directory, const char *spec,
                      const char *password);

char *apk_builder_bind_device(const char *secret);
char *apk_builder_default_key(const char *directory);

char *apk_builder_list_keys(const char *directory);

char *apk_builder_delete_key(const char *path);

char *apk_builder_check_key(const char *path, const char *password);

char *apk_builder_list_projects(const char *directory);

char *apk_builder_project_tree(const char *root);

char *apk_builder_project_health(const char *root);

char *apk_builder_lay_out(const char *name, const char *text);

char *apk_builder_dependencies(const char *root);

char *apk_builder_dependency_remove(const char *root, const char *name);

char *apk_builder_manifest_facts(const char *root);

char *apk_builder_manifest_set(const char *root, const char *field, const char *value);

char *apk_builder_manifest_permission(const char *root, const char *name, bool wanted);

char *apk_builder_symbols(const char *root, const char *needle);

char *apk_builder_where_written(const char *root, const char *qualified);

char *apk_builder_inspect_package(const char *path);

char *apk_builder_check_project(const char *root);

char *apk_builder_search_project(const char *root, const char *needle, bool sensitive,
                          bool whole_word);

char *apk_builder_read_file(const char *root, const char *relative);

char *apk_builder_write_file(const char *root, const char *relative, const char *contents);

char *apk_builder_git_checkout(const char *pack_path, const char *head, const char *into);

char *apk_builder_maven_plan(const char *root);

char *apk_builder_project_repairs(const char *root);

char *apk_builder_apply_repairs(const char *root, const char *repairs);

char *apk_builder_digest_file(const char *path);

char *apk_builder_pack_archive(const char *root, const char *names, const char *into,
                        const char *format, const char *password);

char *apk_builder_adopt_project(const char *root);

char *apk_builder_open_archive(const char *archive_path, const char *into,
                        const char *password);

char *apk_builder_new_folder(const char *root, const char *relative);

char *apk_builder_remove_path(const char *root, const char *relative,
                       const char *trash_root);

char *apk_builder_rename_path(const char *root, const char *from, const char *to);

char *apk_builder_copy_path(const char *root, const char *from, const char *to);

char *apk_builder_build_unsigned(const char *root, const char *package_path,
                          const char *certificate);

char *apk_builder_seal_package(const char *package_path, const char *sealed_path,
                        const char *certificate, const char *signature_v2,
                        const char *signature_v3, const char *signature_v31,
                        uint32_t min_sdk, uint32_t max_sdk);

char *apk_builder_list_built(const char *directory);

char *apk_builder_trash_send(const char *trash_root, const char *path);

char *apk_builder_trash_list(const char *trash_root);

char *apk_builder_trash_restore(const char *trash_root, const char *id);

char *apk_builder_trash_purge(const char *trash_root, const char *id);

char *apk_builder_trash_empty(const char *trash_root);

char *apk_builder_set_icon(const char *root, const char *source);

void apk_builder_string_free(char *value);

}

constexpr uint32_t kExpectedAbiVersion = 1;

#endif
