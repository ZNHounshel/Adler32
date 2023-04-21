import random as rand
import sys

"""
Example use:
python3 data_generation.py "Hello World!"
"""

def generate_string(size_valid: bool, size: int, data_valid: bool, data: str) -> str:
    """Generates the string interpreted by the testbench. In all there are 42 bits"""
    size_valid = "1" if size_valid else "0"
    size = f"{bin(size)[2:]:0>32}"
    data_valid = "1" if data_valid else "0"
    data = f"{bin(ord(data))[2:]:0>8}"
    return f"{size_valid}_{size}_{data_valid}_{data}"

def generate_datafile(filename,data,random_value_chance):
    print(f"Generating data for string: {data}")
    with open(filename, "w") as datafile:
        # Write the string in a comment
        datafile.write(f"# {data}\n")
        # Some random number of values to be read *before* the string that are invalid
        while True:
            random_number = rand.uniform(0, 1)
            if random_number < random_value_chance:
                datafile.write(generate_string(True, len(data), False, "\0") + "\n")
                break
            else:
                datafile.write(generate_string(False, rand.randint(0, 2**32), False, "\0") + "\n")

        # The string, interspresed with random invalid values
        for c in data:
            while True:
                random_number = rand.uniform(0, 1)
                if random_number < random_value_chance:
                    datafile.write(generate_string(False, rand.randint(0, 2**32), True, c) + "\n")
                    break
                else:
                    datafile.write(generate_string(False, rand.randint(0, 2**32), False, chr(rand.randint(65, 122))) + "\n")

if __name__ == "__main__":

    dat = ascii(sys.argv[1]) # Avoid any non-ascii characters. Not actually necessary
    print(f"Generating data for string: {dat}")
    generate_datafile(dat.lower(),dat,0.25)