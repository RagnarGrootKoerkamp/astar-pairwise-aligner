#include "astarpa.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

int main() {
	const char* a = "ACTCGCT";
	const char* b = "AACTCGTT";
	size_t len;
	uint8_t* cigar;
	uint64_t cost;

	cost = astarpa((const uint8_t*)a, strlen(a), (const uint8_t*)b, strlen(b), &cigar, &len);
	assert(cost == 2);
	astarpa_free_cigar(cigar);

	cost = astarpa_gcsh((const uint8_t*)a, strlen(a), (const uint8_t*)b, strlen(b), 1, 15, false,
	                    &cigar, &len);
	assert(cost == 2);
	astarpa_free_cigar(cigar);

	cost =
	    astarpa2_simple((const uint8_t*)a, strlen(a), (const uint8_t*)b, strlen(b), &cigar, &len);
	assert(cost == 2);
	astarpa_free_cigar(cigar);

	cost = astarpa2_full((const uint8_t*)a, strlen(a), (const uint8_t*)b, strlen(b), &cigar, &len);
	assert(cost == 2);
	printf("Cost: %lu\n", cost);
	printf("Cigar len: %lu\n", len);
	printf("Cigar: %s\n", cigar);
	astarpa_free_cigar(cigar);
}
