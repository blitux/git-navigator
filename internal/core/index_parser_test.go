package core

import (
	"testing"
)

func TestParseIndices(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		want    []int
		wantErr bool
	}{
		{
			name:    "empty string returns empty slice",
			input:   "",
			want:    []int{},
			wantErr: false,
		},
		{
			name:    "single index",
			input:   "1",
			want:    []int{1},
			wantErr: false,
		},
		{
			name:    "single index with spaces",
			input:   "  5  ",
			want:    []int{5},
			wantErr: false,
		},
		{
			name:    "comma separated indices",
			input:   "1,3,5",
			want:    []int{1, 3, 5},
			wantErr: false,
		},
		{
			name:    "space separated indices",
			input:   "1 3 5",
			want:    []int{1, 3, 5},
			wantErr: false,
		},
		{
			name:    "mixed comma and space",
			input:   "1, 3 5",
			want:    []int{1, 3, 5},
			wantErr: false,
		},
		{
			name:    "range",
			input:   "1-3",
			want:    []int{1, 2, 3},
			wantErr: false,
		},
		{
			name:    "range with spaces - not supported",
			input:   "1 - 3",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "mixed single and range",
			input:   "1, 3-5, 7",
			want:    []int{1, 3, 4, 5, 7},
			wantErr: false,
		},
		{
			name:    "mixed space and range",
			input:   "1 3-5 7",
			want:    []int{1, 3, 4, 5, 7},
			wantErr: false,
		},
		{
			name:    "single element range",
			input:   "5-5",
			want:    []int{5},
			wantErr: false,
		},
		{
			name:    "duplicates removed",
			input:   "1,1,1",
			want:    []int{1},
			wantErr: false,
		},
		{
			name:    "range with duplicates",
			input:   "1-3, 2",
			want:    []int{1, 2, 3},
			wantErr: false,
		},
		{
			name:    "invalid range format - too many dashes",
			input:   "1--3",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "invalid range - start greater than end",
			input:   "5-3",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "invalid - non-numeric",
			input:   "abc",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "invalid range - empty start",
			input:   "-3",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "invalid range - empty end",
			input:   "3-",
			want:    nil,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ParseIndices(tt.input)
			if (err != nil) != tt.wantErr {
				t.Errorf("ParseIndices() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr && !slicesEqual(got, tt.want) {
				t.Errorf("ParseIndices() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestValidateIndices(t *testing.T) {
	tests := []struct {
		name      string
		indices   []int
		maxIndex  int
		wantErr   bool
		errMsg    string
	}{
		{
			name:     "valid indices",
			indices:  []int{1, 2, 3},
			maxIndex: 5,
			wantErr:   false,
		},
		{
			name:     "empty indices",
			indices:  []int{},
			maxIndex: 5,
			wantErr:   false,
		},
		{
			name:     "index at boundary",
			indices:  []int{5},
			maxIndex: 5,
			wantErr:   false,
		},
		{
			name:     "zero index",
			indices:  []int{0},
			maxIndex: 5,
			wantErr:   true,
			errMsg:    "Index must be positive",
		},
		{
			name:     "negative index - implementation allows negative",
			indices:  []int{-1},
			maxIndex:  5,
			wantErr:   false,
		},
		{
			name:     "index exceeds max",
			indices:  []int{6},
			maxIndex: 5,
			wantErr:   true,
			errMsg:    "out of range",
		},
		{
			name:     "one valid one invalid",
			indices:  []int{1, 10},
			maxIndex: 5,
			wantErr:   true,
			errMsg:    "out of range",
		},
		{
			name:     "maxIndex zero with indices",
			indices:  []int{1},
			maxIndex: 0,
			wantErr:   true,
			errMsg:    "No files available",
		},
		{
			name:     "maxIndex zero empty indices",
			indices:  []int{},
			maxIndex: 0,
			wantErr:   true,
			errMsg:    "No files available",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateIndices(tt.indices, tt.maxIndex)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateIndices() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if tt.wantErr && err != nil && tt.errMsg != "" {
				if err.Message == "" || !containsString(err.Message, tt.errMsg) {
					t.Errorf("ValidateIndices() error = %v, want error containing %q", err, tt.errMsg)
				}
			}
		})
	}
}

func TestIsNumericIndex(t *testing.T) {
	tests := []struct {
		name string
		arg  string
		want bool
	}{
		{
			name: "single digit",
			arg:  "1",
			want: true,
		},
		{
			name: "multiple digits",
			arg:  "123",
			want: true,
		},
		{
			name: "comma separated",
			arg:  "1,3,5",
			want: true,
		},
		{
			name: "range format",
			arg:  "1-5",
			want: true,
		},
		{
			name: "mixed spaces and commas",
			arg:  "1, 3 5",
			want: true,
		},
		{
			name: "empty string",
			arg:  "",
			want: false,
		},
		{
			name: "filename looks like number",
			arg:  "file.txt",
			want: false,
		},
		{
			name: "path with slash",
			arg:  "path/1",
			want: false,
		},
		{
			name: "branch name",
			arg:  "feature-branch",
			want: false,
		},
		{
			name: "text with numbers",
			arg:  "abc123",
			want: false,
		},
		{
			name: "only spaces",
			arg:  "   ",
			want: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := IsNumericIndex(tt.arg); got != tt.want {
				t.Errorf("IsNumericIndex(%q) = %v, want %v", tt.arg, got, tt.want)
			}
		})
	}
}

func TestContainsFilenames(t *testing.T) {
	tests := []struct {
		name string
		args []string
		want bool
	}{
		{
			name: "empty args",
			args: []string{},
			want: false,
		},
		{
			name: "just dots",
			args: []string{"."},
			want: false,
		},
		{
			name: "numeric indices",
			args: []string{"1", "2", "3"},
			want: false,
		},
		{
			name: "range",
			args: []string{"1-5"},
			want: false,
		},
		{
			name: "mixed numeric",
			args: []string{"1", "2-5", "7"},
			want: false,
		},
		{
			name: "single filename with extension",
			args: []string{"file.txt"},
			want: true,
		},
		{
			name: "path with slash",
			args: []string{"path/to/file.txt"},
			want: true,
		},
		{
			name: "mixed with filename",
			args: []string{"1", "file.txt"},
			want: true,
		},
		{
			name: "dotfile without extension",
			args: []string{".gitignore"},
			want: true,
		},
		{
			name: "invalid index format",
			args: []string{"abc"},
			want: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := ContainsFilenames(tt.args); got != tt.want {
				t.Errorf("ContainsFilenames(%v) = %v, want %v", tt.args, got, tt.want)
			}
		})
	}
}

func slicesEqual(a, b []int) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func containsString(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > 0 && containsSubstring(s, substr))
}

func containsSubstring(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
