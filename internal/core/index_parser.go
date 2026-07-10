package core

import (
	"sort"
	"strconv"
	"strings"
)

type IndexRange struct {
	Start int
	End   int
}

func ParseIndices(input string) ([]int, *GitNavigatorError) {
	input = strings.TrimSpace(input)
	if input == "" {
		return []int{}, nil
	}

	indices := make(map[int]bool)

	parts := strings.FieldsFunc(input, func(r rune) bool {
		return r == ' ' || r == ','
	})

	for _, part := range parts {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}

		if strings.Contains(part, "-") {
			rangeParts := strings.Split(part, "-")
			if len(rangeParts) != 2 {
				return nil, InvalidRangeFormat(part)
			}

			start, err := strconv.Atoi(strings.TrimSpace(rangeParts[0]))
			if err != nil {
				return nil, InvalidRangeFormat(part)
			}

			end, err := strconv.Atoi(strings.TrimSpace(rangeParts[1]))
			if err != nil {
				return nil, InvalidRangeFormat(part)
			}

			if start > end {
				return nil, InvalidRangeFormat(part)
			}

			for i := start; i <= end; i++ {
				indices[i] = true
			}
		} else {
			num, err := strconv.Atoi(part)
			if err != nil {
				return nil, InvalidIndexFormat(part)
			}
			indices[num] = true
		}
	}

	result := make([]int, 0, len(indices))
	for idx := range indices {
		result = append(result, idx)
	}
	sort.Ints(result)
	return result, nil
}

func ValidateIndices(indices []int, maxIndex int) *GitNavigatorError {
	if maxIndex == 0 {
		return NoFilesAvailable()
	}

	for _, idx := range indices {
		if idx == 0 {
			return CustomError("Index must be positive (got 0)")
		}
		if idx > maxIndex {
			return IndexOutOfRange(idx, maxIndex)
		}
	}
	return nil
}

func IsNumericIndex(arg string) bool {
	if arg == "" {
		return false
	}
	for _, c := range arg {
		if c != '-' && c != ',' && c != ' ' && (c < '0' || c > '9') {
			return false
		}
	}
	return true
}

func ContainsFilenames(args []string) bool {
	for _, arg := range args {
		if arg == "." {
			continue
		}
		if strings.Contains(arg, ".") && (strings.Contains(arg, "/") || !strings.HasPrefix(arg, ".")) {
			return true
		}
		if _, err := ParseIndices(arg); err != nil {
			return true
		}
	}
	return false
}
