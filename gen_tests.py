import os
import os.path

for file in os.listdir("tests/input"):
    (root, ext) = os.path.splitext(file)

    if os.path.exists("tests/expected_kdl/" + file):
        print(f"test_a_file!(include_str!(\"{'input/' + file}\"), include_str!(\"{'expected_kdl/' + file}\"), {root});")
    else:
        print(f"test_a_file!(fail include_str!(\"{'input/' + file}\"), {root});")