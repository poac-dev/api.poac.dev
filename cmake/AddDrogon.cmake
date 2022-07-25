include_guard(GLOBAL)

message(CHECK_START "Adding drogon")
list(APPEND CMAKE_MESSAGE_INDENT "  ")

include(cmake/CPM.cmake)

CPMAddPackage(
    NAME drogon
    VERSION 1.7.5
    GITHUB_REPOSITORY drogonframework/drogon
    GIT_TAG v1.7.5
)

list(APPEND POAC_API_DEPENDENCIES drogon)
message(CHECK_PASS "added")

list(POP_BACK CMAKE_MESSAGE_INDENT)
