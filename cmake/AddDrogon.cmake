include_guard(GLOBAL)

message(CHECK_START "Adding drogon")
list(APPEND CMAKE_MESSAGE_INDENT "  ")

include(cmake/CPM.cmake)

if (APPLE)
    set(OPENSSL_ROOT_DIR "/opt/homebrew/opt/openssl")
endif ()

CPMAddPackage(
    NAME drogon
    VERSION 1.7.5
    GITHUB_REPOSITORY drogonframework/drogon
    GIT_TAG c9f5946eff9d74175357642100001efb4298e301
    OPTIONS
        "OPENSSL_ROOT_DIR ${OPENSSL_ROOT_DIR}"
)

list(APPEND POAC_API_DEPENDENCIES drogon)
message(CHECK_PASS "added")

list(POP_BACK CMAKE_MESSAGE_INDENT)
