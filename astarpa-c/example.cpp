#include "astarpa.h"

#include <cassert>
#include <iostream>
#include <string>

int main() {
	const std::string a = "ACTCGCT";
	const std::string b = "AACTCGTT";
	size_t len;
	uint8_t* cigar;
	size_t cost = astarpa((const uint8_t*)a.c_str(), a.size(), (const uint8_t*)b.c_str(), b.size(),
	                      &cigar, &len);
	std::string cigar_string = (const char*)cigar;
	assert(cost == 2);
	assert(cigar_string == "=I4=X=");
	astarpa_free_cigar(cigar);
	std::cout << "Cost: " << cost << std::endl;
	std::cout << "Cigar len: " << len << std::endl;
	std::cout << "Cigar: " << cigar_string << std::endl;
}
